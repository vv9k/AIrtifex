use airtifex_api::config::Config;
use airtifex_api::id::V1Context as ClockContext;
use airtifex_api::llm::{llama, InferenceRequest, ModelName};
use airtifex_api::models::{llm::LargeLanguageModel, user::User};
use airtifex_api::routes::{api, r#static};
use airtifex_api::{DbPool, Error, InnerAppState, Result, SharedAppState};
use airtifex_core::user::AccountType;

use axum::Router;
use axum_extra::extract::cookie::Key;
use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::Level;
use tracing::Span;
use tracing_subscriber::{field::MakeExt, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("Feature postgres and sqlite are mutually exclusive and cannot be enabled together");

#[derive(Debug, Parser)]
#[command(version = "0.1.0", about = "")]
pub struct Opts {
    #[command(subcommand)]
    /// Subcommand to run
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    Serve,
}

async fn initialize_models(
    db: Arc<DbPool>,
    config: &Config,
    runtime: Arc<Runtime>,
) -> Result<HashMap<ModelName, flume::Sender<InferenceRequest>>> {
    let mut txs = HashMap::new();
    for (model, config) in config.llms.iter() {
        let exists = LargeLanguageModel::get_by_name(&db, &model).await.is_ok();

        log::info!("initializing model {model}, exists in db: {exists}");

        if !exists {
            let llm = LargeLanguageModel::new(model.clone(), config.model_description.clone());
            llm.create(&db).await?;
        }
        let tx_inference_req = llama::initialize_model_and_handle_inferences(
            db.clone(),
            config.clone(),
            runtime.clone(),
        );
        txs.insert(model.clone(), tx_inference_req);
    }
    Ok(txs)
}

async fn inner(runtime: Arc<Runtime>) -> Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "airtifex_api=debug,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .map_fmt_fields(|f| f.display_messages())
                .event_format(tracing_subscriber::fmt::format::Format::default()),
        )
        .init();

    let config = Config::read("./config.yaml")?;

    let opts = Opts::parse();

    match opts.command {
        Command::Serve => {
            let db_pool = Arc::new(
                DbPool::connect(&config.db_url)
                    .await
                    .map_err(Error::DatabaseError)?,
            );

            #[cfg(feature = "sqlite")]
            {
                airtifex_api::models::run_pragma(&db_pool).await?;
                let migrations = sqlx::migrate!("migrations/sqlite").run(&*db_pool).await?;
                tracing::debug!("{:?} {:?}", migrations, db_pool);
            }

            #[cfg(feature = "postgres")]
            {
                let migrations = sqlx::migrate!("migrations/postgres").run(&*db_pool).await?;
                tracing::debug!("{:?} {:?}", migrations, db_pool);
            }

            let context = ClockContext::new(0);

            let user = User::new("admin", "admin", "", AccountType::Admin);
            let _ = user.create(&db_pool).await;

            let listen = (config.listen_addr.clone(), config.listen_port.clone());

            let tx_inference_req =
                initialize_models(db_pool.clone(), &config, runtime.clone()).await?;

            std::env::set_var("JWT_SECRET", &config.jwt_secret);

            let app = Router::new()
                .merge(api::router())
                .merge(r#static::router())
                .with_state(SharedAppState::from(Arc::new(InnerAppState {
                    db: db_pool,
                    uuid_context: context,
                    key: Key::generate(),
                    config,
                    tx_inference_req,
                })))
                .layer(
                    tower_http::trace::TraceLayer::new_for_http()
                        .make_span_with(
                            tower_http::trace::DefaultMakeSpan::new().level(Level::INFO),
                        )
                        .on_failure(
                            |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {},
                        )
                        .on_response(
                            |rsp: &axum::response::Response, latency: Duration, _span: &Span| {
                                tracing::info!("{} {}ms", rsp.status(), latency.as_millis());
                            },
                        ),
                );

            tracing::info!("listening on {}:{}", listen.0, listen.1);
            Ok(axum::Server::bind(&listen.into())
                .serve(app.into_make_service())
                .await?)
        }
    }
}

fn main() {
    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(8) // adjust the number of worker threads as needed
            .enable_all()
            .build()
            .unwrap(),
    );
    let rt = runtime.clone();
    runtime.block_on(async move {
        if let Err(e) = inner(rt).await {
            eprintln!("Execution failed - {}", e);
        }
    })
}
