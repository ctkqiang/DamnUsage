use crate::application::commands::configure_provider::{
    ConfigureProviderCommand, ConfigureProviderHandler,
};
use crate::application::commands::remove_provider::{RemoveProviderCommand, RemoveProviderHandler};
use crate::application::commands::sync_usage::{
    SyncAllProvidersCommand, SyncProviderUsageCommand, SyncUsageHandler,
};
use crate::application::commands::transition_notch::{
    TransitionNotchCommand, TransitionNotchHandler,
};
use crate::application::commands::update_preferences::{
    UpdatePreferencesCommand, UpdatePreferencesHandler,
};
use crate::application::projections::usage_projector::UsageProjector;
use crate::application::queries::get_dashboard::{
    DashboardReadModel, GetDashboardHandler, GetDashboardQuery,
};
use crate::application::queries::get_preferences::{GetPreferencesHandler, GetPreferencesQuery};
use crate::application::queries::get_provider_detail::{
    GetProviderDetailHandler, GetProviderDetailQuery, ProviderDetailReadModel,
};
use crate::application::queries::list_providers::{
    ListProviderCatalogHandler, ListProviderCatalogQuery, ProviderCatalogItemView,
};
use crate::domain::entities::{ProviderAccount, UsageSnapshot, UserPreferences};
use crate::domain::errors::DomainResult;
use crate::domain::ports::{
    EventPublisherPort, NotchWindowPort, SecretVaultPort, UsageProviderPort, WriteRepositoryPort,
};
use crate::domain::value_objects::{NotchDisplayState, ProviderKind};
use std::collections::HashMap;
use std::sync::Arc;

pub struct CommandBus {
    configure_provider: ConfigureProviderHandler,
    remove_provider: RemoveProviderHandler,
    sync_usage: SyncUsageHandler,
    update_preferences: UpdatePreferencesHandler,
    transition_notch: TransitionNotchHandler,
}

impl CommandBus {
    pub fn new(
        repository: Arc<dyn WriteRepositoryPort>,
        vault: Arc<dyn SecretVaultPort>,
        providers: HashMap<ProviderKind, Arc<dyn UsageProviderPort>>,
        demo_provider: Arc<dyn UsageProviderPort>,
        window_port: Arc<dyn NotchWindowPort>,
        projector: Arc<UsageProjector>,
        publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            configure_provider: ConfigureProviderHandler::new(
                repository.clone(),
                vault.clone(),
                projector.clone(),
                publisher.clone(),
            ),
            remove_provider: RemoveProviderHandler::new(
                repository.clone(),
                vault.clone(),
                projector.clone(),
                publisher.clone(),
            ),
            sync_usage: SyncUsageHandler::new(
                repository.clone(),
                vault,
                providers,
                demo_provider,
                projector.clone(),
                publisher.clone(),
            ),
            update_preferences: UpdatePreferencesHandler::new(
                repository,
                projector.clone(),
                publisher.clone(),
            ),
            transition_notch: TransitionNotchHandler::new(window_port, projector, publisher),
        }
    }

    pub async fn configure_provider(
        &self,
        cmd: ConfigureProviderCommand,
    ) -> DomainResult<ProviderAccount> {
        self.configure_provider.handle(cmd).await
    }

    pub async fn remove_provider(&self, cmd: RemoveProviderCommand) -> DomainResult<()> {
        self.remove_provider.handle(cmd).await
    }

    pub async fn sync_provider(
        &self,
        cmd: SyncProviderUsageCommand,
    ) -> DomainResult<UsageSnapshot> {
        self.sync_usage.handle_one(cmd).await
    }

    pub async fn sync_all(&self, cmd: SyncAllProvidersCommand) -> DomainResult<Vec<UsageSnapshot>> {
        self.sync_usage.handle_all(cmd).await
    }

    pub async fn update_preferences(
        &self,
        cmd: UpdatePreferencesCommand,
    ) -> DomainResult<UserPreferences> {
        self.update_preferences.handle(cmd).await
    }

    pub async fn transition_notch(
        &self,
        cmd: TransitionNotchCommand,
    ) -> DomainResult<NotchDisplayState> {
        self.transition_notch.handle(cmd).await
    }
}

pub struct QueryBus {
    get_dashboard: GetDashboardHandler,
    get_provider_detail: GetProviderDetailHandler,
    list_catalog: ListProviderCatalogHandler,
    get_preferences: GetPreferencesHandler,
}

impl QueryBus {
    pub fn new(projector: Arc<UsageProjector>) -> Self {
        Self {
            get_dashboard: GetDashboardHandler::new(projector.clone()),
            get_provider_detail: GetProviderDetailHandler::new(projector.clone()),
            list_catalog: ListProviderCatalogHandler::new(projector.clone()),
            get_preferences: GetPreferencesHandler::new(projector),
        }
    }

    pub async fn get_dashboard(&self, query: GetDashboardQuery) -> DashboardReadModel {
        self.get_dashboard.handle(query).await
    }

    pub async fn get_provider_detail(
        &self,
        query: GetProviderDetailQuery,
    ) -> DomainResult<ProviderDetailReadModel> {
        self.get_provider_detail.handle(query).await
    }

    pub async fn list_catalog(&self, query: ListProviderCatalogQuery) -> Vec<ProviderCatalogItemView> {
        self.list_catalog.handle(query).await
    }

    pub async fn get_preferences(&self, query: GetPreferencesQuery) -> UserPreferences {
        self.get_preferences.handle(query).await
    }
}

pub struct CqrsApp {
    pub commands: CommandBus,
    pub queries: QueryBus,
    repository: Arc<dyn WriteRepositoryPort>,
    projector: Arc<UsageProjector>,
}

impl CqrsApp {
    pub fn new(
        repository: Arc<dyn WriteRepositoryPort>,
        vault: Arc<dyn SecretVaultPort>,
        providers: HashMap<ProviderKind, Arc<dyn UsageProviderPort>>,
        demo_provider: Arc<dyn UsageProviderPort>,
        window_port: Arc<dyn NotchWindowPort>,
        publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        let projector = Arc::new(UsageProjector::new(UserPreferences::default()));
        let commands = CommandBus::new(
            repository.clone(),
            vault,
            providers,
            demo_provider,
            window_port,
            projector.clone(),
            publisher,
        );
        let queries = QueryBus::new(projector.clone());

        Self {
            commands,
            queries,
            repository,
            projector,
        }
    }

    pub async fn bootstrap(&self) -> DomainResult<()> {
        let preferences = self.repository.load_preferences().await?;
        let accounts = self.repository.list_accounts().await?;
        let events = self.repository.load_events().await?;

        self.projector
            .hydrate(accounts, events, preferences.clone())
            .await;

        if !self.projector.has_accounts().await && preferences.demo_mode {
            self.seed_demo_accounts().await?;
        }

        let _ = self
            .commands
            .sync_all(SyncAllProvidersCommand {
                include_demo_if_empty: preferences.demo_mode,
            })
            .await;

        Ok(())
    }

    async fn seed_demo_accounts(&self) -> DomainResult<()> {
        let defaults = [
            (ProviderKind::Antigravity, "Antigravity (AGY)", 60.0),
            (ProviderKind::ClaudeCode, "Claude Code", 100.0),
            (ProviderKind::OpenRouter, "OpenRouter", 80.0),
            (ProviderKind::DeepSeek, "DeepSeek", 30.0),
            (ProviderKind::AmazonBedrock, "Amazon Bedrock", 120.0),
            (ProviderKind::OpenAi, "OpenAI", 75.0),
            (ProviderKind::MoonshotKimi, "Moonshot Kimi", 25.0),
            (ProviderKind::SiliconFlow, "SiliconFlow", 20.0),
        ];

        for (kind, label, budget) in defaults {
            let mut metadata = HashMap::new();
            metadata.insert("demo".to_string(), "true".to_string());
            let mut credentials = HashMap::new();
            credentials.insert("demo_seed".to_string(), "active".to_string());
            for field in crate::domain::catalog::catalog_entry_for(kind).credential_fields {
                if field.required {
                    credentials.insert(field.key, "demo_placeholder".to_string());
                }
            }
            let _ = self
                .commands
                .configure_provider(ConfigureProviderCommand {
                    provider_id: Some(format!("demo-{}", kind.as_str())),
                    kind,
                    label: label.to_string(),
                    enabled: true,
                    monthly_budget_usd: Some(budget),
                    endpoint_override: None,
                    credentials,
                    metadata,
                })
                .await;
        }
        Ok(())
    }
}
