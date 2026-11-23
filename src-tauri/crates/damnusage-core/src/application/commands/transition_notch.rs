use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::errors::DomainResult;
use crate::domain::events::{DomainEvent, DomainEventEnvelope};
use crate::domain::ports::{EventPublisherPort, NotchWindowPort};
use crate::domain::value_objects::NotchDisplayState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionNotchCommand {
    pub target_state: NotchDisplayState,
}

pub struct TransitionNotchHandler {
    window_port: Arc<dyn NotchWindowPort>,
    projector: Arc<UsageProjector>,
    publisher: Arc<dyn EventPublisherPort>,
}

impl TransitionNotchHandler {
    pub fn new(
        window_port: Arc<dyn NotchWindowPort>,
        projector: Arc<UsageProjector>,
        publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            window_port,
            projector,
            publisher,
        }
    }

    pub async fn handle(&self, cmd: TransitionNotchCommand) -> DomainResult<NotchDisplayState> {
        self.window_port.apply_state(cmd.target_state).await?;

        let envelope = DomainEventEnvelope::wrap(DomainEvent::NotchStateChanged {
            state: cmd.target_state,
        });
        self.projector.apply_event(&envelope).await;
        self.publisher.publish(envelope).await;

        Ok(cmd.target_state)
    }
}
