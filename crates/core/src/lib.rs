use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentMode {
    Observation,
    Assisted,
    Automatic,
    Exception,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    User,
    Agent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Success,
    Error,
    Skipped,
    Waiting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Focus,
    Input,
    Select,
    Click,
    Confirm,
    Navigate,
    Wait,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: String,
    pub location: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub index: u32,
    pub timestamp: DateTime<Utc>,
    pub action: ActionKind,
    pub target: String,
    pub value: Option<String>,
    pub actor: Actor,
    pub confidence: Option<f32>,
    pub expected: Option<Value>,
    pub observed: Option<Value>,
    pub status: StepStatus,
    pub evidence: Option<Evidence>,
    pub sensitive: bool,
}

impl Step {
    pub fn new(index: u32, action: ActionKind, target: impl Into<String>, actor: Actor) -> Self {
        let target = target.into();
        Self {
            index,
            timestamp: Utc::now(),
            action,
            sensitive: is_sensitive_target(&target),
            target,
            value: None,
            actor,
            confidence: None,
            expected: None,
            observed: None,
            status: StepStatus::Waiting,
            evidence: None,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.value = Some(if self.sensitive {
            "[REDACTED]".to_string()
        } else {
            value
        });
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub version: u32,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub final_confirmation_required: bool,
}

impl Workflow {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: 1,
            description: None,
            created_at: Utc::now(),
            final_confirmation_required: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub mode: AgentMode,
    pub steps: Vec<Step>,
    pub final_confirmation_required: bool,
}

impl Session {
    pub fn new(workflow: &Workflow, mode: AgentMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id: workflow.id,
            started_at: Utc::now(),
            completed_at: None,
            mode,
            steps: Vec::new(),
            final_confirmation_required: workflow.final_confirmation_required,
        }
    }

    pub fn push_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
    }
}

pub fn is_sensitive_target(target: &str) -> bool {
    let normalized = target.to_ascii_lowercase().replace(['-', ' '], "_");
    [
        "password",
        "passwd",
        "token",
        "secret",
        "api_key",
        "authorization",
        "credit_card",
        "card_number",
        "cvv",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_values_are_redacted() {
        let step = Step::new(1, ActionKind::Input, "api-token", Actor::User)
            .with_value("super-secret");
        assert!(step.sensitive);
        assert_eq!(step.value.as_deref(), Some("[REDACTED]"));
    }

    #[test]
    fn normal_values_are_preserved() {
        let step = Step::new(1, ActionKind::Input, "track_title", Actor::User)
            .with_value("Stoned");
        assert!(!step.sensitive);
        assert_eq!(step.value.as_deref(), Some("Stoned"));
    }
}
