use recall_core::{ActionKind, AgentMode, Step};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EngineConfig {
    pub assisted_after_successes: u32,
    pub automatic_after_successes: u32,
    pub minimum_confidence: f32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            assisted_after_successes: 1,
            automatic_after_successes: 3,
            minimum_confidence: 0.90,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionDecision {
    Observe,
    Suggest,
    Execute,
    RequireUserConfirmation,
    Halt,
}

#[derive(Debug, Clone)]
pub struct WorkflowEngine {
    config: EngineConfig,
    mode: AgentMode,
    successful_sessions: u32,
    mode_before_pause: AgentMode,
}

impl WorkflowEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            mode: AgentMode::Observation,
            successful_sessions: 0,
            mode_before_pause: AgentMode::Observation,
        }
    }

    pub fn mode(&self) -> AgentMode {
        self.mode
    }

    pub fn successful_sessions(&self) -> u32 {
        self.successful_sessions
    }

    pub fn record_successful_session(&mut self) {
        self.successful_sessions += 1;
        self.mode = if self.successful_sessions >= self.config.automatic_after_successes {
            AgentMode::Automatic
        } else if self.successful_sessions >= self.config.assisted_after_successes {
            AgentMode::Assisted
        } else {
            AgentMode::Observation
        };
    }

    pub fn record_error(&mut self) {
        self.mode = AgentMode::Exception;
    }

    pub fn observe_user_correction(&mut self) {
        self.mode = AgentMode::Observation;
        self.successful_sessions = 0;
    }

    pub fn pause(&mut self) {
        if self.mode != AgentMode::Paused {
            self.mode_before_pause = self.mode;
            self.mode = AgentMode::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.mode == AgentMode::Paused {
            self.mode = self.mode_before_pause;
        }
    }

    pub fn decide(&mut self, step: &Step) -> ExecutionDecision {
        if self.mode == AgentMode::Paused || self.mode == AgentMode::Exception {
            return ExecutionDecision::Halt;
        }

        if is_final_action(step) {
            return ExecutionDecision::RequireUserConfirmation;
        }

        let confidence = step.confidence.unwrap_or(0.0);
        match self.mode {
            AgentMode::Observation => ExecutionDecision::Observe,
            AgentMode::Assisted => ExecutionDecision::Suggest,
            AgentMode::Automatic if confidence >= self.config.minimum_confidence => {
                ExecutionDecision::Execute
            }
            AgentMode::Automatic => {
                self.record_error();
                ExecutionDecision::Halt
            }
            AgentMode::Exception | AgentMode::Paused => ExecutionDecision::Halt,
        }
    }
}

fn is_final_action(step: &Step) -> bool {
    let target = step.target.to_ascii_lowercase();
    matches!(&step.action, ActionKind::Confirm)
        || ["publish", "release", "submit", "delete", "purchase"]
            .iter()
            .any(|needle| target.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use recall_core::{Actor, Step};

    fn safe_step(confidence: f32) -> Step {
        let mut step = Step::new(1, ActionKind::Input, "title", Actor::Agent);
        step.confidence = Some(confidence);
        step
    }

    #[test]
    fn promotes_only_after_repeated_success() {
        let mut engine = WorkflowEngine::new(EngineConfig::default());
        assert_eq!(engine.mode(), AgentMode::Observation);
        engine.record_successful_session();
        assert_eq!(engine.mode(), AgentMode::Assisted);
        engine.record_successful_session();
        assert_eq!(engine.mode(), AgentMode::Assisted);
        engine.record_successful_session();
        assert_eq!(engine.mode(), AgentMode::Automatic);
    }

    #[test]
    fn low_confidence_in_automatic_mode_halts() {
        let mut engine = WorkflowEngine::new(EngineConfig::default());
        for _ in 0..3 {
            engine.record_successful_session();
        }
        assert_eq!(engine.decide(&safe_step(0.40)), ExecutionDecision::Halt);
        assert_eq!(engine.mode(), AgentMode::Exception);
    }

    #[test]
    fn publication_always_requires_a_human() {
        let mut engine = WorkflowEngine::new(EngineConfig::default());
        for _ in 0..3 {
            engine.record_successful_session();
        }
        let mut step = Step::new(9, ActionKind::Click, "publish_track", Actor::Agent);
        step.confidence = Some(1.0);
        assert_eq!(
            engine.decide(&step),
            ExecutionDecision::RequireUserConfirmation
        );
    }
}
