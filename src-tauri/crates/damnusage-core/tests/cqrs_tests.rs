use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use damnusage_core::application::bus::CqrsApp;
use damnusage_core::application::commands::configure_provider::ConfigureProviderCommand;
use damnusage_core::application::commands::remove_provider::RemoveProviderCommand;
use damnusage_core::application::commands::transition_notch::TransitionNotchCommand;
use damnusage_core::application::commands::update_preferences::UpdatePreferencesCommand;
use damnusage_core::application::queries::get_dashboard::GetDashboardQuery;
use damnusage_core::application::queries::get_preferences::GetPreferencesQuery;
use damnusage_core::application::queries::get_provider_detail::GetProviderDetailQuery;
use damnusage_core::application::queries::list_providers::ListProviderCatalogQuery;
use damnusage_core::domain::errors::DomainResult;
use damnusage_core::domain::events::DomainEventEnvelope;
use damnusage_core::domain::ports::{EventPublisherPort, NotchWindowPort};
use damnusage_core::domain::value_objects::{
    CurrencyCode, LocaleCode, NotchDisplayState, ProviderKind,
};
use damnusage_core::infrastructure::persistence::json_event_store::JsonEventStore;
use damnusage_core::infrastructure::persistence::secret_vault::FileSecretVault;
use damnusage_core::infrastructure::providers::aws_sigv4::{sign_aws_request, SigV4Input};
use damnusage_core::infrastructure::providers::demo_simulator::DemoUsageSimulator;
use damnusage_core::infrastructure::providers::registry::build_provider_registry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct RecordingPublisher {
    events: Mutex<Vec<DomainEventEnvelope>>,
}

impl RecordingPublisher {
    fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl EventPublisherPort for RecordingPublisher {
    async fn publish(&self, event: DomainEventEnvelope) {
        self.events.lock().await.push(event);
    }
}

struct NoopNotchWindow;

#[async_trait]
impl NotchWindowPort for NoopNotchWindow {
    async fn apply_state(&self, _state: NotchDisplayState) -> DomainResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn cqrs_bootstrap_seeds_demo_and_projects_dashboard() {
    let publisher = Arc::new(RecordingPublisher::new());
    let app = CqrsApp::new(
        Arc::new(JsonEventStore::in_memory()),
        Arc::new(FileSecretVault::in_memory()),
        build_provider_registry(),
        Arc::new(DemoUsageSimulator::new()),
        Arc::new(NoopNotchWindow),
        publisher.clone(),
    );

    app.bootstrap().await.expect("bootstrap succeeds");

    let dashboard = app.queries.get_dashboard(GetDashboardQuery).await;
    assert_eq!(dashboard.providers.len(), 8);
    assert!(dashboard.total_spend_usd > 100.0);
    assert!(dashboard.total_tokens > 0);
    assert!(dashboard.headline.active_provider_count == 8);

    let catalog = app.queries.list_catalog(ListProviderCatalogQuery).await;
    assert_eq!(catalog.len(), 20);
    assert!(catalog
        .iter()
        .any(|c| c.entry.kind == ProviderKind::AmazonBedrock));
    assert!(catalog
        .iter()
        .any(|c| c.entry.kind == ProviderKind::OpenRouter));
    assert!(catalog
        .iter()
        .any(|c| c.entry.kind == ProviderKind::Antigravity));
    assert!(catalog
        .iter()
        .any(|c| c.entry.kind == ProviderKind::ClaudeCode));
    assert!(catalog
        .iter()
        .any(|c| c.entry.kind == ProviderKind::DeepSeek));
}

#[tokio::test]
async fn cqrs_commands_update_read_projections_and_emit_events() {
    let publisher = Arc::new(RecordingPublisher::new());
    let app = CqrsApp::new(
        Arc::new(JsonEventStore::in_memory()),
        Arc::new(FileSecretVault::in_memory()),
        build_provider_registry(),
        Arc::new(DemoUsageSimulator::new()),
        Arc::new(NoopNotchWindow),
        publisher.clone(),
    );

    let prefs = app
        .commands
        .update_preferences(UpdatePreferencesCommand {
            locale: Some(LocaleCode::ZhCn),
            currency: Some(CurrencyCode::Cny),
            demo_mode: Some(false),
            ..Default::default()
        })
        .await
        .expect("preferences update succeeds");
    assert_eq!(prefs.locale, LocaleCode::ZhCn);
    assert_eq!(prefs.currency, CurrencyCode::Cny);

    let read_prefs = app.queries.get_preferences(GetPreferencesQuery).await;
    assert_eq!(read_prefs.locale, LocaleCode::ZhCn);

    let mut creds = HashMap::new();
    creds.insert("api_key".to_string(), "sk-or-v1-test".to_string());
    let account = app
        .commands
        .configure_provider(ConfigureProviderCommand {
            provider_id: None,
            kind: ProviderKind::OpenRouter,
            label: "Primary OpenRouter".to_string(),
            enabled: true,
            monthly_budget_usd: Some(120.0),
            endpoint_override: None,
            credentials: creds,
            metadata: HashMap::new(),
        })
        .await
        .expect("configure provider succeeds");

    let detail = app
        .queries
        .get_provider_detail(GetProviderDetailQuery {
            provider_id: account.id.clone(),
        })
        .await
        .expect("detail exists");
    assert_eq!(detail.account.label, "Primary OpenRouter");

    app.commands
        .transition_notch(TransitionNotchCommand {
            target_state: NotchDisplayState::Expanded,
        })
        .await
        .expect("notch transition succeeds");

    let dashboard = app.queries.get_dashboard(GetDashboardQuery).await;
    assert_eq!(dashboard.notch_state, NotchDisplayState::Expanded);

    app.commands
        .remove_provider(RemoveProviderCommand {
            provider_id: account.id.clone(),
        })
        .await
        .expect("remove provider succeeds");

    assert!(app
        .queries
        .get_provider_detail(GetProviderDetailQuery {
            provider_id: account.id
        })
        .await
        .is_err());

    assert!(publisher.events.lock().await.len() >= 4);
}

#[test]
fn aws_sigv4_produces_valid_authorization_header() {
    let ts = Utc.with_ymd_and_hms(2026, 5, 1, 12, 0, 0).unwrap();
    let signed = sign_aws_request(&SigV4Input {
        method: "POST",
        service: "ce",
        region: "us-east-1",
        host: "ce.us-east-1.amazonaws.com",
        canonical_uri: "/",
        canonical_query: "",
        extra_headers: &[
            ("content-type", "application/x-amz-json-1.1"),
            ("x-amz-target", "AWSInsightsIndexService.GetCostAndUsage"),
        ],
        payload: b"{}",
        access_key_id: "AKIAIOSFODNN7EXAMPLE",
        secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        session_token: None,
        timestamp: ts,
    });

    assert_eq!(signed.amz_date, "20260501T120000Z");
    assert!(signed
        .authorization
        .starts_with("AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20260501/us-east-1/ce/aws4_request"));
    assert!(signed.authorization.contains("Signature="));
}
