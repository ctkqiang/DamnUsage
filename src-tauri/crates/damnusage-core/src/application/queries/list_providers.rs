use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::value_objects::ProviderCatalogEntry;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListProviderCatalogQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCatalogItemView {
    pub entry: ProviderCatalogEntry,
    pub configured_count: usize,
    pub configured_ids: Vec<String>,
}

pub struct ListProviderCatalogHandler {
    projector: Arc<UsageProjector>,
}

impl ListProviderCatalogHandler {
    pub fn new(projector: Arc<UsageProjector>) -> Self {
        Self { projector }
    }

    pub async fn handle(&self, _query: ListProviderCatalogQuery) -> Vec<ProviderCatalogItemView> {
        self.projector.read_catalog().await
    }
}
