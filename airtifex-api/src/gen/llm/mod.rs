use crate::{
    config::{Config, LlmConfig},
    gen::ModelName,
    models::llm::LargeLanguageModel,
    DbPool, Result,
};

use std::{collections::HashMap, sync::Arc};
use tokio::runtime::Runtime;

pub mod inference;

pub use inference::*;

pub async fn initialize_models(
    db: Arc<DbPool>,
    config: &Config,
    runtime: Arc<Runtime>,
) -> Result<HashMap<ModelName, (LlmConfig, flume::Sender<InferenceRequest>)>> {
    let mut txs = HashMap::new();
    for (model, llm_config) in config.llms.iter() {
        let exists = LargeLanguageModel::get_by_name(&db, model).await.is_ok();

        log::info!("initializing model {model}, exists in db: {exists}");

        if !exists {
            let llm =
                LargeLanguageModel::new(model.to_owned(), llm_config.model_description.clone());
            llm.create(&db).await?;
        }
        let tx_inference_req = inference::initialize_model_and_handle_inferences(
            model.to_owned(),
            db.clone(),
            llm_config.clone(),
            runtime.clone(),
        );
        txs.insert(model.clone(), (llm_config.clone(), tx_inference_req));
    }
    Ok(txs)
}
