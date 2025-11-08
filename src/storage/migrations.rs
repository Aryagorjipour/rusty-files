use crate::core::error::{Result, SearchError};
use crate::storage::schema;
use chrono::Utc;
use rusqlite::Connection;

pub struct MigrationManager;

impl MigrationManager {
    pub fn initialize_schema(conn: &Connection) -> Result<()> {
        for pragma in schema::OPTIMIZE_PRAGMAS {
            conn.execute(pragma, [])?;
        }

        conn.execute(schema::CREATE_SCHEMA_VERSION_TABLE, [])?;

        let current_version = Self::get_current_version(conn)?;

        if current_version == 0 {
            Self::apply_initial_schema(conn)?;
        } else if current_version < schema::CURRENT_SCHEMA_VERSION {
            Self::migrate(conn, current_version, schema::CURRENT_SCHEMA_VERSION)?;
        } else if current_version > schema::CURRENT_SCHEMA_VERSION {
            return Err(SearchError::IndexCorrupted(format!(
                "Database schema version {} is newer than supported version {}",
                current_version, schema::CURRENT_SCHEMA_VERSION
            )));
        }

        Ok(())
    }

    fn get_current_version(conn: &Connection) -> Result<i32> {
        let version: rusqlite::Result<i32> = conn.query_row(
            "SELECT MAX(version) FROM schema_version",
            [],
            |row| row.get(0),
        );

        match version {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(SearchError::Database(e)),
        }
    }

    fn apply_initial_schema(conn: &Connection) -> Result<()> {
        let tx = conn.unchecked_transaction()?;

        for statement in schema::get_all_table_creation_statements() {
            tx.execute(statement, [])?;
        }

        for statement in schema::get_all_index_creation_statements() {
            tx.execute(statement, [])?;
        }

        tx.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            [schema::CURRENT_SCHEMA_VERSION.to_string(), Utc::now().to_rfc3339()],
        )?;

        tx.commit()?;

        Ok(())
    }

    fn migrate(conn: &Connection, from: i32, to: i32) -> Result<()> {
        for version in from..to {
            Self::apply_migration(conn, version, version + 1)?;
        }
        Ok(())
    }

    fn apply_migration(conn: &Connection, _from: i32, to: i32) -> Result<()> {
        let tx = conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
            [to.to_string(), Utc::now().to_rfc3339()],
        )?;

        tx.commit()?;

        Ok(())
    }

    pub fn verify_schema(conn: &Connection) -> Result<bool> {
        let current_version = Self::get_current_version(conn)?;
        Ok(current_version == schema::CURRENT_SCHEMA_VERSION)
    }

    pub fn rebuild_indexes(conn: &Connection) -> Result<()> {
        let tx = conn.unchecked_transaction()?;

        for statement in schema::get_all_index_creation_statements() {
            let drop_statement = statement.replace("CREATE INDEX IF NOT EXISTS", "DROP INDEX IF EXISTS");
            let drop_statement = drop_statement.split(" ON ").next().unwrap_or("");

            if !drop_statement.is_empty() {
                let _ = tx.execute(drop_statement, []);
            }

            tx.execute(statement, [])?;
        }

        tx.commit()?;

        Ok(())
    }
}
