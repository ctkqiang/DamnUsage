use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::entities::{ProviderAccount, UsageSnapshot};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{ProviderCatalogEntry, SparklinePoint};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProviderDetailQuery {
    pub provider_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDetailReadModel {
    pub account: ProviderAccount,
    pub catalog: ProviderCatalogEntry,
    pub snapshot: Option<UsageSnapshot>,
    pub history: Vec<SparklinePoint>,
}

pub struct GetProviderDetailHandler {
    projector: Arc<UsageProjector>,
}

impl GetProviderDetailHandler {
    pub fn new(projector: Arc<UsageProjector>) -> Self {
        Self { projector }
    }

    pub async fn handle(&self, query: GetProviderDetailQuery) -> DomainResult<ProviderDetailReadModel> {
        self.projector
            .read_provider_detail(&query.provider_id)
            .await
            .ok_or_else(|| DomainError::ProviderNotFound(query.provider_id))
    }
}
