use crate::config::Config;
use crate::gen::ModelName;
use crate::models::llm::LargeLanguageModel;
use crate::{DbPool, Result};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub mod llama;

pub use llama::*;

pub async fn initialize_models(
    db: Arc<DbPool>,
    config: &Config,
    runtime: Arc<Runtime>,
) -> Result<HashMap<ModelName, flume::Sender<InferenceRequest>>> {
    let mut txs = HashMap::new();
    for (model, llm_config) in config.llms.iter() {
        let exists = LargeLanguageModel::get_by_name(&db, &model).await.is_ok();

        log::info!("initializing model {model}, exists in db: {exists}");

        if !exists {
            let llm = LargeLanguageModel::new(model.clone(), llm_config.model_description.clone());
            llm.create(&db).await?;
        }
        let tx_inference_req = llama::initialize_model_and_handle_inferences(
            db.clone(),
            llm_config.clone(),
            runtime.clone(),
        );
        txs.insert(model.clone(), tx_inference_req);
    }
    Ok(txs)
}
