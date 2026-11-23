use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::entities::UserPreferences;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetPreferencesQuery;

pub struct GetPreferencesHandler {
    projector: Arc<UsageProjector>,
}

impl GetPreferencesHandler {
    pub fn new(projector: Arc<UsageProjector>) -> Self {
        Self { projector }
    }

    pub async fn handle(&self, _query: GetPreferencesQuery) -> UserPreferences {
        self.projector.read_preferences().await
    }
}
