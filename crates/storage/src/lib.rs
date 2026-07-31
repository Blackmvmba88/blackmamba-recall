use recall_core::{Session, Workflow};
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn migrate(&self) -> Result<(), StorageError> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version INTEGER NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                mode TEXT NOT NULL,
                payload TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                FOREIGN KEY(workflow_id) REFERENCES workflows(id)
            );

            CREATE TABLE IF NOT EXISTS steps (
                session_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                status TEXT NOT NULL,
                payload TEXT NOT NULL,
                PRIMARY KEY(session_id, step_index),
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            "#,
        )?;
        Ok(())
    }

    pub fn save_workflow(&self, workflow: &Workflow) -> Result<(), StorageError> {
        let payload = serde_json::to_string(workflow)?;
        self.conn.execute(
            r#"
            INSERT INTO workflows (id, name, version, payload, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
              name = excluded.name,
              version = excluded.version,
              payload = excluded.payload
            "#,
            params![
                workflow.id.to_string(),
                workflow.name,
                workflow.version,
                payload,
                workflow.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn save_session(&mut self, session: &Session) -> Result<(), StorageError> {
        let tx = self.conn.transaction()?;
        let payload = serde_json::to_string(session)?;
        tx.execute(
            r#"
            INSERT INTO sessions (id, workflow_id, mode, payload, started_at, completed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
              mode = excluded.mode,
              payload = excluded.payload,
              completed_at = excluded.completed_at
            "#,
            params![
                session.id.to_string(),
                session.workflow_id.to_string(),
                format!("{:?}", session.mode).to_ascii_lowercase(),
                payload,
                session.started_at.to_rfc3339(),
                session
                    .completed_at
                    .as_ref()
                    .map(|value| value.to_rfc3339()),
            ],
        )?;

        tx.execute(
            "DELETE FROM steps WHERE session_id = ?1",
            params![session.id.to_string()],
        )?;

        for step in &session.steps {
            tx.execute(
                "INSERT INTO steps (session_id, step_index, status, payload) VALUES (?1, ?2, ?3, ?4)",
                params![
                    session.id.to_string(),
                    step.index,
                    format!("{:?}", step.status).to_ascii_lowercase(),
                    serde_json::to_string(step)?,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_session(&self, id: Uuid) -> Result<Option<Session>, StorageError> {
        let mut statement = self
            .conn
            .prepare("SELECT payload FROM sessions WHERE id = ?1")?;
        let mut rows = statement.query(params![id.to_string()])?;
        match rows.next()? {
            Some(row) => {
                let payload: String = row.get(0)?;
                Ok(Some(serde_json::from_str(&payload)?))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use recall_core::{ActionKind, Actor, AgentMode, Session, Step, StepStatus};

    #[test]
    fn round_trips_a_session() {
        let workflow = Workflow::new("soundcloud_upload");
        let mut session = Session::new(&workflow, AgentMode::Observation);
        let mut step = Step::new(1, ActionKind::Input, "title", Actor::User)
            .with_value("Stoned");
        step.status = StepStatus::Success;
        session.push_step(step);

        let mut store = Store::in_memory().expect("store");
        store.save_workflow(&workflow).expect("workflow");
        store.save_session(&session).expect("session");
        let loaded = store
            .load_session(session.id)
            .expect("load")
            .expect("present");
        assert_eq!(loaded.steps[0].value.as_deref(), Some("Stoned"));
    }
}
