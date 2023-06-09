use airtifex_api::{
    config::Config,
    gen,
    id::V1Context as ClockContext,
    models::user::User,
    routes::{api, r#static},
    DbPool, Error, InnerAppState, Result, SharedAppState,
};
use airtifex_core::user::AccountType;

use axum::{extract::DefaultBodyLimit, Router};
use axum_extra::extract::cookie::Key;
use clap::Parser;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::runtime::Runtime;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::{Level, Span};
use tracing_subscriber::{field::MakeExt, layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("Feature postgres and sqlite are mutually exclusive and cannot be enabled together");

#[derive(Debug, Parser)]
#[command(version = "0.1.0", about = "")]
pub struct Opts {
    #[arg(short, long)]
    #[clap(default_value = "config.yaml")]
    pub config: PathBuf,
    #[command(subcommand)]
    /// Subcommand to run
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    Serve,
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

    let opts = Opts::parse();

    let config = Config::read(&opts.config)?;

    match opts.command {
        Command::Serve => {
            let db_pool = Arc::new(
                DbPool::connect(&config.db_url)
                    .await
                    .map_err(Error::DatabaseError)?,
            );

            #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
            {
                airtifex_api::models::run_pragma(&db_pool).await?;
                sqlx::migrate!("migrations/sqlite").run(&*db_pool).await?;
            }

            #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
            {
                sqlx::migrate!("migrations/postgres").run(&*db_pool).await?;
            }

            let context = ClockContext::new(0);

            let user = User::new("admin", "admin", "", AccountType::Admin);
            let _ = user.create(&db_pool).await;

            let listen = (config.listen_addr, config.listen_port);

            let tx_inference_req =
                gen::llm::initialize_models(db_pool.clone(), &config, runtime.clone()).await?;
            let tx_image_gen_req =
                gen::image::initialize_models(db_pool.clone(), &config, runtime.clone()).await?;

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
                    tx_image_gen_req,
                })))
                .layer(DefaultBodyLimit::max(8 * 1000 * 1000))
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
