//! Meeting chat. Chime carries a message to whoever is connected at that moment;
//! this table is what keeps it, so it is still there on the next join.
use super::{lobby, mint_id};
use crate::{
    db,
    models::{meeting_message as message, meeting_visit as visit},
    rahti::{Result, Validator},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::Serialize;

/// The most a participant is sent on joining.
const HISTORY: u64 = 200;

#[derive(Serialize)]
pub struct Message {
    pub id: String,
    pub name: String,
    pub text: String,
    pub at: i64,
}
impl From<message::Model> for Message {
    fn from(row: message::Model) -> Self {
        Self {
            id: row.id,
            name: row.sender_name,
            text: row.body,
            at: row.sent_at,
        }
    }
}

/// Only someone in the call reads or writes its chat: a guest still waiting in
/// the lobby holds a ticket too.
async fn participant(id: &str, ticket: &str) -> Result<visit::Model> {
    let row = lobby::mine(id, ticket).await?;
    if row.state != "joined" {
        return Err(lobby::denied());
    }
    Ok(row)
}

pub async fn history(id: String, ticket: String) -> Result<Vec<Message>> {
    participant(&id, &ticket).await?;
    let mut rows = message::Entity::find()
        .filter(message::Column::MeetingId.eq(&id))
        .order_by_desc(message::Column::SentAt)
        .order_by_desc(message::Column::Id)
        .limit(HISTORY)
        .all(db::db())
        .await?;
    rows.reverse();
    Ok(rows.into_iter().map(Message::from).collect())
}

/// The sender's name comes from their visit, never from the request.
pub async fn send(id: String, ticket: String, text: String) -> Result<Message> {
    let text = text.trim().to_string();
    let mut validator = Validator::new();
    validator
        .required("text", &text)
        .max_chars("text", &text, 500);
    validator.finish()?;
    let sender = participant(&id, &ticket).await?;
    let row = message::ActiveModel {
        id: Set(mint_id()),
        meeting_id: Set(id),
        sender_id: Set(sender.user_id),
        sender_name: Set(sender.name),
        body: Set(text),
        sent_at: Set(rahti::time::Timestamp::now().as_millisecond()),
    }
    .insert(db::db())
    .await?;
    Ok(row.into())
}

/// A revoked link takes its conversation with it.
pub(super) async fn clear(id: &str) -> Result<()> {
    message::Entity::delete_many()
        .filter(message::Column::MeetingId.eq(id))
        .exec(db::db())
        .await?;
    Ok(())
}
