use super::*;

/// The migrations that ran in `setup` created a real schema: a row goes in
/// through the entity and comes back. A disagreement between an entity and
/// its migration fails here, on the first query, which is the whole reason
/// this test is separate from the ones that go through the router — nothing
/// checks the two against each other at build time.
#[tokio::test]
async fn the_meeting_link_table_round_trips() {
    use sea_orm::ActiveValue::Set;
    use sea_orm::{ActiveModelTrait, EntityTrait};

    setup().await;

    let created_at = crate::rahti::time::Utc::now();

    let saved = crate::models::meeting_link::ActiveModel {
        id: Set("roundtripcode01".to_string()),
        owner_id: Set(None),
        title: Set("A link written straight to the table".to_string()),
        created_at: Set(created_at),
        revoked_at: Set(None),
        provider_meeting_id: Set(None),
        media_region: Set(None),
    }
    .insert(crate::db::db())
    .await
    .expect("the insert");

    let found = crate::models::meeting_link::Entity::find_by_id(&saved.id)
        .one(crate::db::db())
        .await
        .expect("the query")
        .expect("the row");

    assert_eq!(found.title, "A link written straight to the table");
    assert_eq!(found.revoked_at, None);
    assert_eq!(found.provider_meeting_id, None);
}

/// The columns the meeting provider writes are nullable and are written
/// after the row exists, so the update path is worth its own query: a column
/// the migration made `NOT NULL` by accident fails here rather than the
/// first time somebody joins a meeting.
#[tokio::test]
async fn a_conference_can_be_recorded_against_a_link() {
    use sea_orm::ActiveValue::{Set, Unchanged};
    use sea_orm::{ActiveModelTrait, EntityTrait};

    setup().await;

    let saved = crate::models::meeting_link::ActiveModel {
        id: Set("conferencecode1".to_string()),
        owner_id: Set(None),
        title: Set("A link that gets a conference".to_string()),
        created_at: Set(crate::rahti::time::Utc::now()),
        revoked_at: Set(None),
        provider_meeting_id: Set(None),
        media_region: Set(None),
    }
    .insert(crate::db::db())
    .await
    .expect("the insert");

    crate::models::meeting_link::ActiveModel {
        id: Unchanged(saved.id.clone()),
        provider_meeting_id: Set(Some("chime-meeting-id".to_string())),
        media_region: Set(Some("us-east-1".to_string())),
        ..Default::default()
    }
    .update(crate::db::db())
    .await
    .expect("the update");

    let found = crate::models::meeting_link::Entity::find_by_id(&saved.id)
        .one(crate::db::db())
        .await
        .expect("the query")
        .expect("the row");

    assert_eq!(
        found.provider_meeting_id.as_deref(),
        Some("chime-meeting-id")
    );
    assert_eq!(found.media_region.as_deref(), Some("us-east-1"));
}
