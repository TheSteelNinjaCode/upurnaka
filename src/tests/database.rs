use super::*;

/// The migrations that ran in `setup` created a real schema: a row goes in
/// through the entity and comes back. A disagreement between the entity and
/// its migration fails here, on the first query.
#[tokio::test]
async fn the_todo_table_round_trips() {
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};

    setup().await;

    let saved = crate::models::todo::ActiveModel {
        text: Set("write a test".to_string()),
        done: Set(false),
        ..Default::default()
    }
    .insert(crate::db::db())
    .await
    .expect("the insert");

    let found = crate::models::todo::Entity::find_by_id(saved.id)
        .one(crate::db::db())
        .await
        .expect("the query")
        .expect("the row");

    assert_eq!(found.text, "write a test");
}
