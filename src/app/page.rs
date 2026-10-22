//! The window opens where the desktop client opens: on Chat.
//!
//! A page may answer with any compatible Axum response, so the root is a
//! redirect rather than a second copy of the chat UI — one URL owns that
//! screen, and the rail's selected item stays truthful.

use axum::response::Redirect;

pub async fn page() -> Redirect {
    Redirect::temporary("/chat")
}
