//! The `todo` table.
//!
//! One file per table, named after it in snake_case. `src/models/mod.rs` is
//! generated from this directory, so saving the file is the whole of adding
//! the entity — there is no list to register it in.
//!
//! Copy this file to add a table. The four pieces are always the same:
//!
//! 1. `Model` — the columns, as ordinary Rust fields.
//! 2. `Relation` — what this table points at. Empty is fine and common.
//! 3. `ActiveModelBehavior` — hooks. The empty impl is required.
//! 4. a migration in `src/migrations/`, which is what actually creates it.
//!
//! The struct does not create the table. Nothing here is read at startup and
//! nothing is checked against the database: an entity that disagrees with the
//! schema compiles fine and fails on the first query. The migration is the
//! schema; this is how Rust talks to it.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// `Serialize` so an `#[rpc]` can return a row as it stands, and the page can
/// seed a list into its script with `Json(&todos)`. `Deserialize` so the same
/// shape round-trips back.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "todo")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub text: String,
    pub done: bool,
}

/// What this table points at. A table with no foreign keys has none, and the
/// empty enum is how that is written.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

/// Hooks for insert and update. Required even when empty — this is where a
/// `created_at` would be filled in.
impl ActiveModelBehavior for ActiveModel {}
