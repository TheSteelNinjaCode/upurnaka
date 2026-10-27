//! This application, driven through its generated router.
//!
//! Add one module per cohesive feature area, not one file per test. Shared
//! setup and request helpers stay in `support.rs`. See
//! docs/conventions/testing.md.

mod support;
use support::*;

mod auth;
mod calendar;
mod database;
pub(crate) mod giphy;
mod meeting_controls;
mod meetings;
mod pages;
mod people;
mod recordings;
mod rpc;
mod settings;
mod users;
