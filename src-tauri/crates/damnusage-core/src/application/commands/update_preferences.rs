use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::entities::UserPreferences;
use crate::domain::errors::DomainResult;
use crate::domain::events::{DomainEvent, DomainEventEnvelope};
use crate::domain::ports::{EventPublisherPort, WriteRepositoryPort};
use crate::domain::value_objects::{CurrencyCode, LocaleCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdatePreferencesCommand {
    pub locale: Option<LocaleCode>,
    pub currency: Option<CurrencyCode>,
    pub usd_to_cny_rate: Option<f64>,
    pub refresh_interval_secs: Option<u64>,
    pub monthly_global_budget_usd: Option<f64>,
    pub warning_threshold_pct: Option<f64>,
    pub critical_threshold_pct: Option<f64>,
    pub demo_mode: Option<bool>,
    pub hover_to_peek: Option<bool>,
    pub show_cost_in_notch: Option<bool>,
    pub notch_width_offset_px: Option<i32>,
}

pub struct UpdatePreferencesHandler {
    repository: Arc<dyn WriteRepositoryPort>,
    projector: Arc<UsageProjector>,
    publisher: Arc<dyn EventPublisherPort>,
}

impl UpdatePreferencesHandler {
    pub fn new(
        repository: Arc<dyn WriteRepositoryPort>,
        projector: Arc<UsageProjector>,
        publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            repository,
            projector,
            publisher,
        }
    }

    pub async fn handle(&self, cmd: UpdatePreferencesCommand) -> DomainResult<UserPreferences> {
        let mut prefs = self.repository.load_preferences().await?;

        if let Some(locale) = cmd.locale {
            prefs.locale = locale;
        }
        if let Some(currency) = cmd.currency {
            prefs.currency = currency;
        }
        if let Some(rate) = cmd.usd_to_cny_rate {
            if rate > 0.1 {
                prefs.usd_to_cny_rate = rate;
            }
        }
        if let Some(interval) = cmd.refresh_interval_secs {
            prefs.refresh_interval_secs = interval.clamp(10, 3600);
        }
        if let Some(budget) = cmd.monthly_global_budget_usd {
            if budget >= 1.0 {
                prefs.monthly_global_budget_usd = budget;
            }
        }
        if let Some(warn) = cmd.warning_threshold_pct {
            prefs.warning_threshold_pct = warn.clamp(10.0, 99.0);
        }
        if let Some(crit) = cmd.critical_threshold_pct {
            prefs.critical_threshold_pct = crit.clamp(prefs.warning_threshold_pct, 100.0);
        }
        if let Some(demo) = cmd.demo_mode {
            prefs.demo_mode = demo;
        }
        if let Some(hover) = cmd.hover_to_peek {
            prefs.hover_to_peek = hover;
        }
        if let Some(show_cost) = cmd.show_cost_in_notch {
            prefs.show_cost_in_notch = show_cost;
        }
        if let Some(offset) = cmd.notch_width_offset_px {
            prefs.notch_width_offset_px = offset.clamp(-80, 80);
        }

        self.repository.save_preferences(prefs.clone()).await?;

        let envelope = DomainEventEnvelope::wrap(DomainEvent::PreferencesUpdated {
            preferences: prefs.clone(),
        });
        self.repository.append_event(envelope.clone()).await?;
        self.projector.apply_event(&envelope).await;
        self.publisher.publish(envelope).await;

        Ok(prefs)
    }
}
