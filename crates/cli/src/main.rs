use anyhow::Result;
use clap::{Parser, Subcommand};
use recall_core::{ActionKind, Actor, AgentMode, Session, Step, StepStatus, Workflow};
use recall_storage::Store;
use recall_workflow_engine::{EngineConfig, WorkflowEngine};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "recall", version, about = "BlackMamba Recall control CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create or migrate the local SQLite database.
    InitDb {
        #[arg(long, default_value = "recall.db")]
        path: PathBuf,
    },
    /// Generate a safe, offline SoundCloud upload simulation.
    DemoSoundcloud {
        #[arg(long, default_value = "recall.db")]
        path: PathBuf,
    },
    /// Show promotion through observation, assisted and automatic modes.
    StateDemo,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::InitDb { path } => {
            Store::open(&path)?;
            println!("Database ready: {}", path.display());
        }
        Command::DemoSoundcloud { path } => run_soundcloud_demo(path)?,
        Command::StateDemo => run_state_demo(),
    }
    Ok(())
}

fn run_soundcloud_demo(path: PathBuf) -> Result<()> {
    let mut workflow = Workflow::new("soundcloud_upload");
    workflow.description = Some(
        "Offline simulation only. The final publish action always remains manual.".into(),
    );

    let mut session = Session::new(&workflow, AgentMode::Observation);

    let mut title = Step::new(1, ActionKind::Input, "track_title", Actor::User)
        .with_value("Stoned");
    title.status = StepStatus::Success;
    session.push_step(title);

    let mut genre = Step::new(2, ActionKind::Select, "genre", Actor::Agent)
        .with_value("Alternative Rock");
    genre.confidence = Some(0.94);
    genre.status = StepStatus::Success;
    session.push_step(genre);

    let mut publish = Step::new(3, ActionKind::Click, "publish_track", Actor::Agent);
    publish.confidence = Some(0.99);
    publish.status = StepStatus::Waiting;
    session.push_step(publish);

    let mut engine = WorkflowEngine::new(EngineConfig::default());
    for _ in 0..3 {
        engine.record_successful_session();
    }
    let decision = engine.decide(session.steps.last().expect("publish step"));

    let mut store = Store::open(&path)?;
    store.save_workflow(&workflow)?;
    store.save_session(&session)?;

    println!("{}", serde_json::to_string_pretty(&session)?);
    println!("Final action decision: {decision:?}");
    println!("No browser action was performed.");
    Ok(())
}

fn run_state_demo() {
    let mut engine = WorkflowEngine::new(EngineConfig::default());
    println!("0 successes -> {:?}", engine.mode());
    for index in 1..=3 {
        engine.record_successful_session();
        println!("{index} successes -> {:?}", engine.mode());
    }
}
