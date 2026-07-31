use recall_core::AgentMode;
use recall_workflow_engine::{EngineConfig, WorkflowEngine};

fn main() {
    let engine = WorkflowEngine::new(EngineConfig::default());
    let visible_state = match engine.mode() {
        AgentMode::Observation => "OBSERVING",
        AgentMode::Assisted => "ASSISTED",
        AgentMode::Automatic => "AUTOMATIC",
        AgentMode::Exception => "EXCEPTION",
        AgentMode::Paused => "PAUSED",
    };

    println!("BlackMamba Recall desktop shell");
    println!("Agent state: {visible_state}");
    println!("No global keyboard hooks or browser actions are enabled in this milestone.");
}
