//! The database connection.
//!
//! One connection for the process, reached from anywhere by `db()`. Rahti
//! handlers take no state — `page()` and an `#[rpc]` are ordinary functions —
//! so a pool cannot arrive as an argument, and `DatabaseConnection` is an
//! internally-pooled handle anyway: cloning it is cheap and sharing it is the
//! intended use.
//!
//! This file is yours. If you want a pool size, a statement timeout, or a
//! read replica, this is where they go.

// A connection you have not queried yet is not dead code, and a new project
// has not queried it yet — the same reason `src/models/mod.rs` carries this.
#![allow(dead_code)]

use std::sync::OnceLock;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

static DB: OnceLock<DatabaseConnection> = OnceLock::new();

/// The connection, from a page, an rpc, or a route handler.
///
/// Panics if the process has not connected yet, which can only happen by
/// calling this before `main` reaches `connect()`. That is a wiring mistake
/// rather than a runtime condition, and a message naming it is worth more
/// than an `Option` every call site has to unwrap.
pub fn db() -> &'static DatabaseConnection {
    DB.get().expect(
        "the database has not been connected.\n  \
         `db::connect_if_configured().await` runs from `main` before the \
         server starts — \
         if you removed it, put it back.",
    )
}

/// Whether a connection has been installed for this process.
pub fn is_connected() -> bool {
    DB.get().is_some()
}

/// Connect once at startup when a database has been configured.
///
/// An absent `DATABASE_URL` leaves the application online without persistence,
/// so deployment owners can set their chosen connection string later. A present
/// but unusable URL remains an error instead of silently hiding a broken setup.
pub async fn connect_if_configured() -> Result<(), String> {
    // One `.env` reader for the process, in the framework — `AuthSettings`
    // needs the same file, and a project with no database still has one.
    // Idempotent, so neither caller has to know which ran first, and a real
    // environment variable still wins over the file.
    rahti::load_env();

    let Some(url) = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(());
    };

    connect_to(&url).await.map_err(|e| {
        format!(
            "cannot connect to the database: {e}\n  \
             Check DATABASE_URL, and that the server is running."
        )
    })
}

/// Connect to a named database instead of DATABASE_URL.
///
/// This is how a test gets one. SQLite needs no server, so a whole suite can
/// run against a real engine:
///
/// ```ignore
/// # async fn example() {
/// let path = std::env::temp_dir().join("my-app-tests.db");
/// let _ = std::fs::remove_file(&path);
///
/// crate::db::connect_to(&format!("sqlite://{}?mode=rwc", path.display()))
///     .await
///     .unwrap();
/// crate::db::migrate().await.unwrap();
/// # }
/// ```
///
/// Use a file, not `:memory:`. The connection here is a process-wide static,
/// but every `#[tokio::test]` builds its own runtime and drops it at the end
/// of the test — so the pool that opened the database belongs to a runtime
/// that is gone by the time the next test runs. An in-memory database exists
/// only while something is connected to it, so it goes with that runtime and
/// every later test fails with `no such table` on a table the migration
/// certainly created. A file survives being reconnected to.
///
/// Delete it on the way in rather than on the way out: a test binary has no
/// reliable teardown, and starting from nothing is what makes a run
/// repeatable.
///
/// Connecting a second time is a no-op rather than an error. The connection
/// belongs to the process, and tests run in parallel within one.
pub async fn connect_to(url: &str) -> Result<(), sea_orm::DbErr> {
    if DB.get().is_some() {
        return Ok(());
    }

    let mut options = ConnectOptions::new(url.to_string());
    options.sqlx_logging(false);

    // Keep one connection open for the life of the process.
    //
    // A warm connection is a small win on any backend and the difference
    // between working and not on an in-memory SQLite: that database exists
    // only as long as something is connected to it, so a pool that drains to
    // zero between two queries takes the schema with it. The symptom is `no
    // such table` on a table the migration definitely created.
    options.min_connections(1);

    let connection = Database::connect(options).await?;
    let _ = DB.set(connection);
    Ok(())
}

/// Apply every pending migration.
///
/// Deliberately not called by `connect`. A framework that changes your schema
/// because you started the server is the same surprise as one that downloads
/// a compiler you did not ask for — and the version of that surprise which
/// happens in production is much worse. Call it from a `main` branch behind
/// an argument, or run it from a test, when you want it.
///
/// ```ignore
/// # async fn example() {
/// crate::db::migrate().await.unwrap();
/// # }
/// ```
pub async fn migrate() -> Result<(), String> {
    use sea_orm_migration::MigratorTrait;

    crate::migrations::Migrator::up(db(), None)
        .await
        .map_err(|e| format!("a migration failed: {e}"))
}
