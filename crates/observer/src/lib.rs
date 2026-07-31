use chrono::{DateTime, Utc};
use recall_core::{ActionKind, Actor, Step};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationEvent {
    pub timestamp: DateTime<Utc>,
    pub action: ActionKind,
    pub target: String,
    pub value: Option<String>,
}

impl ObservationEvent {
    pub fn into_step(self, index: u32) -> Step {
        let mut step = Step::new(index, self.action, self.target, Actor::User);
        if let Some(value) = self.value {
            step = step.with_value(value);
        }
        step
    }
}

#[derive(Debug, Clone, Default)]
pub struct PauseSwitch {
    paused: Arc<AtomicBool>,
}

impl PauseSwitch {
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
}

pub trait ObservationSink {
    fn record(&mut self, event: ObservationEvent) -> Result<(), ObserverError>;
}

#[derive(Debug, Error)]
pub enum ObserverError {
    #[error("observation is paused")]
    Paused,
    #[error("sensitive target rejected: {0}")]
    SensitiveTarget(String),
    #[error("observer backend error: {0}")]
    Backend(String),
}

#[derive(Debug, Default)]
pub struct MemoryObserver {
    pause_switch: PauseSwitch,
    events: Vec<ObservationEvent>,
}

impl MemoryObserver {
    pub fn pause_switch(&self) -> PauseSwitch {
        self.pause_switch.clone()
    }

    pub fn events(&self) -> &[ObservationEvent] {
        &self.events
    }
}

impl ObservationSink for MemoryObserver {
    fn record(&mut self, event: ObservationEvent) -> Result<(), ObserverError> {
        if self.pause_switch.is_paused() {
            return Err(ObserverError::Paused);
        }
        self.events.push(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_switch_stops_recording() {
        let mut observer = MemoryObserver::default();
        observer.pause_switch().pause();
        let result = observer.record(ObservationEvent {
            timestamp: Utc::now(),
            action: ActionKind::Click,
            target: "save".into(),
            value: None,
        });
        assert!(matches!(result, Err(ObserverError::Paused)));
    }
}
