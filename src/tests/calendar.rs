use super::*;
use rahti::serde_json::{Value, json};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

#[tokio::test]
async fn calendar_month_picker_browses_selects_and_dismisses() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/calendar"))
        .await
        .unwrap();
    browser
        .click("button[aria-label='Choose calendar date']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#popover-calendar-date').matches(':popover-open') && document.querySelectorAll('.calendar-picker-range').length === 7 && document.querySelectorAll('.calendar-picker-selected').length === 1 && document.activeElement.classList.contains('calendar-picker-day')").await.unwrap();
    let initial: String = browser
        .eval("document.querySelector('.calendar-picker-header h2').textContent")
        .await
        .unwrap();
    let _: bool = browser
        .eval("document.querySelector('button[aria-label=\"Next month\"]').click(); true")
        .await
        .unwrap();
    browser
        .wait_for(&format!(
            "document.querySelector('.calendar-picker-header h2').textContent !== {}",
            rahti::serde_json::to_string(&initial).unwrap()
        ))
        .await
        .unwrap();
    let next_month: String = browser
        .eval("document.querySelector('.calendar-picker-header h2').textContent")
        .await
        .unwrap();
    let _: bool = browser
        .eval("document.querySelector('button[aria-label=\"Previous month\"]').click(); true")
        .await
        .unwrap();
    browser
        .wait_for(&format!(
            "document.querySelector('.calendar-picker-header h2').textContent === {}",
            rahti::serde_json::to_string(&initial).unwrap()
        ))
        .await
        .unwrap();
    let _: bool = browser
        .eval("document.querySelector('button[aria-label=\"Next month\"]').click(); true")
        .await
        .unwrap();
    browser
        .wait_for(&format!(
            "document.querySelector('.calendar-picker-header h2').textContent === {}",
            rahti::serde_json::to_string(&next_month).unwrap()
        ))
        .await
        .unwrap();
    // Pick a Sunday in the browsed month. Work week must show the following
    // Monday, while the picker highlights this Sunday-to-Saturday row.
    let chosen: String = browser.eval("(() => { const dates = Array.from(document.querySelectorAll('.calendar-picker-day')); const sunday = dates.find((date,index) => index % 7 === 0 && !date.classList.contains('calendar-picker-outside')); const key = sunday.value; sunday.click(); return key; })()").await.unwrap();
    browser
        .wait_for("!document.querySelector('#popover-calendar-date').matches(':popover-open')")
        .await
        .unwrap();
    browser
        .click("button[aria-label='Choose calendar date']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('#popover-calendar-date').matches(':popover-open') && document.activeElement.classList.contains('calendar-picker-day')").await.unwrap();
    let selected: String = browser
        .eval("document.querySelector('.calendar-picker-selected').value")
        .await
        .unwrap();
    // Sunday's Work week starts on Monday; the selected week remains visible.
    assert_eq!(
        selected, chosen,
        "the chosen Sunday remains selected in its highlighted week"
    );
    let start: String = browser
        .eval("document.querySelector('.calendar-grid-slot').value.split('@')[0]")
        .await
        .unwrap();
    let monday: bool = browser
        .eval(&format!("new Date('{start}T12:00').getDay() === 1"))
        .await
        .unwrap();
    assert!(monday);
    let adjacent: bool = browser
        .eval(&format!(
            "(new Date('{start}T12:00') - new Date('{chosen}T12:00')) / 86400000 === 1"
        ))
        .await
        .unwrap();
    assert!(
        adjacent,
        "the calendar displays the work week directly after the chosen Sunday"
    );
    browser.press("ArrowRight").await.unwrap();
    browser.wait_for("document.activeElement.classList.contains('calendar-picker-day') && document.activeElement.getAttribute('aria-label') !== document.querySelector('.calendar-picker-selected').getAttribute('aria-label')").await.unwrap();
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("!document.querySelector('#popover-calendar-date').matches(':popover-open')")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

#[tokio::test]
async fn calendar_time_marker_and_half_hour_selection_follow_the_visible_days() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/calendar"))
        .await
        .unwrap();
    browser.wait_for("document.querySelectorAll('.calendar-grid-slot').length === 240 && document.querySelectorAll('.calendar-slot-selected').length === 1").await.unwrap();
    let aligned: bool = browser.eval("(() => { const line = document.querySelector('.calendar-now'); if (line.hidden) return true; const segment = document.querySelector('.calendar-now-today').getBoundingClientRect(); const grid = line.parentElement.getBoundingClientRect(); const slot = document.querySelector('.calendar-slot-selected'); const half = Number(slot.value.split('@')[1]); return Math.abs(segment.width - grid.width / 5) < 1 && Math.floor(parseFloat(line.style.top) / 28) === half && getComputedStyle(slot).outlineWidth === '2px'; })()").await.unwrap();
    assert!(
        aligned,
        "the current-time segment and outlined half-hour align to today's column"
    );
    let _: bool = browser
        .eval("document.querySelector('main button[aria-label=Next]').click(); true")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.calendar-now').hidden")
        .await
        .unwrap();
    let selected: String = browser.eval("(() => { const slot = document.querySelector('.calendar-grid-slot[value$=\"@18\"]'); slot.scrollIntoView({block:'center'}); return slot.value; })()").await.unwrap();
    browser
        .click(".calendar-grid-slot[value$='@18']")
        .await
        .unwrap();
    let date = selected.split('@').next().unwrap();
    browser.wait_for("!document.querySelector('.calendar-editor').hidden && document.activeElement.name === 'title'").await.unwrap();
    let click_times: bool = browser.eval(&format!("document.querySelector('input[name=start_date]').value === '{date}' && document.querySelector('input[name=start_time]').value === '09:00' && document.querySelector('input[name=end_time]').value === '09:30'")).await.unwrap();
    assert!(
        click_times,
        "one click opens the meeting editor at the clicked time"
    );
    browser.press("Escape").await.unwrap();
    browser.wait_for("document.querySelector('.calendar-editor').hidden && document.activeElement.textContent.includes('New meeting')").await.unwrap();
    let _: bool = browser
        .eval("document.querySelector('.calendar-slot-selected').focus(); true")
        .await
        .unwrap();
    browser.wait_for(&format!("document.activeElement.value === '{selected}' && document.activeElement.getAttribute('aria-pressed') === 'true'")).await.unwrap();
    browser.press("ArrowDown").await.unwrap();
    browser.wait_for(&format!("document.activeElement.value === '{date}@19' && document.activeElement.getAttribute('aria-pressed') === 'true'")).await.unwrap();
    browser.press("Enter").await.unwrap();
    browser.wait_for("!document.querySelector('.calendar-editor').hidden && document.activeElement.name === 'title'").await.unwrap();
    let correct: bool = browser.eval(&format!("document.querySelector('input[name=start_date]').value === '{date}' && document.querySelector('input[name=start_time]').value === '09:30' && document.querySelector('input[name=end_time]').value === '10:00'")).await.unwrap();
    assert!(correct, "the meeting editor uses the selected half-hour");
    browser.press("Escape").await.unwrap();
    browser
        .wait_for("document.querySelector('.calendar-editor').hidden")
        .await
        .unwrap();
    let _: bool = browser
        .eval("document.querySelector('button[aria-label=\"Calendar view\"]').click(); true")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('#dropdown-menu-calendar-view').matches(':popover-open')")
        .await
        .unwrap();
    let _: bool = browser.eval("Array.from(document.querySelectorAll('#dropdown-menu-calendar-view [role=menuitem]')).find(item => item.textContent.trim() === 'Day').click(); true").await.unwrap();
    browser.wait_for("document.querySelectorAll('.calendar-grid-slot').length === 48 && !document.querySelector('.calendar-now').hidden").await.unwrap();
    let full_day: bool = browser.eval("(() => { const segment = document.querySelector('.calendar-now-today').getBoundingClientRect(); const grid = document.querySelector('.calendar-now').parentElement.getBoundingClientRect(); return Math.abs(segment.width - grid.width) < 1 && Math.abs(segment.left - grid.left) < 1; })()").await.unwrap();
    assert!(
        full_day,
        "Day view keeps the time marker within its single column"
    );
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}

fn input(title: &str) -> Value {
    json!({"title":title,"attendees":"Ada, grace@example.com","start_date":"2026-10-27","start_time":"11:00","end_date":"2026-10-27","end_time":"11:30","timezone":"America/Managua","all_day":false,"recurrence":"none","occurrences":1,"location":"Room 2","details":"Agenda and notes"})
}
async fn call(value: Value) -> (StatusCode, Value) {
    let (status, body) = rpc(
        "/calendar",
        "schedule_meeting",
        &json!({"input":value}).to_string(),
    )
    .await;
    (status, rahti::serde_json::from_str(&body).unwrap())
}

#[tokio::test]
async fn scheduling_persists_utc_times_details_and_private_join_links() {
    let (status, body) = call(input("Persistent planning")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let id = body[0]["event_id"].as_str().unwrap();
    let row = crate::models::calendar_event::Entity::find_by_id(id)
        .one(crate::db::db())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.starts_at.to_rfc3339(), "2026-10-27T17:00:00+00:00");
    assert_eq!(row.details, "Agenda and notes");
    assert!(row.attendees.contains("grace@example.com"));
    assert_eq!(
        crate::meetings::find(id).await.unwrap().unwrap().title,
        "Persistent planning"
    );
    let (_, html) = get("/calendar").await;
    assert!(html.contains("Persistent planning"));
    let mut other = super::auth::Client::new().await;
    other.with_cookie(super::support::cookie_for(
        "other-user",
        "Other User",
        "other@example.com",
    ));
    assert!(
        !other
            .request("/calendar", None)
            .await
            .2
            .contains("Persistent planning")
    );
    let mut anonymous = super::auth::Client::new().await;
    assert_eq!(
        anonymous
            .request(
                "/calendar",
                Some(("schedule_meeting", json!({"input":input("Unauthorized")})))
            )
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        rpc("/calendar", "schedule_meeting", r#"{"input":{}}"#)
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn recurrence_all_day_and_invalid_ranges_are_checked_before_writing() {
    setup().await;
    for (field, value) in [
        ("end_time", json!("10:00")),
        ("timezone", json!("Bad/Zone")),
        ("recurrence", json!("yearly")),
        ("occurrences", json!(53)),
        ("attendees", json!("bad@")),
        ("details", json!("x".repeat(8001))),
    ] {
        let mut payload = input("Invalid calendar event");
        payload[field] = value;
        assert_eq!(
            call(payload).await.0,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field}"
        );
    }
    assert_eq!(
        crate::models::calendar_event::Entity::find()
            .filter(crate::models::calendar_event::Column::Title.eq("Invalid calendar event"))
            .count(crate::db::db())
            .await
            .unwrap(),
        0
    );
    let mut payload = input("Weekly calendar event");
    payload["recurrence"] = json!("weekly");
    payload["occurrences"] = json!(3);
    let (status, body) = call(payload).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let ids: std::collections::HashSet<_> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["event_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 3);
    assert_eq!(
        crate::models::calendar_event::Entity::find()
            .filter(crate::models::calendar_event::Column::Title.eq("Weekly calendar event"))
            .count(crate::db::db())
            .await
            .unwrap(),
        3
    );
    let mut payload = input("All-day calendar event");
    payload["all_day"] = json!(true);
    payload["timezone"] = json!("UTC");
    let (status, body) = call(payload).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body[0]["length"], json!(24.0));
    let mut payload = input("DST invalid calendar event");
    payload["timezone"] = json!("America/New_York");
    payload["start_date"] = json!("2026-03-08");
    payload["end_date"] = json!("2026-03-08");
    payload["start_time"] = json!("02:30");
    payload["end_time"] = json!("03:30");
    assert_eq!(call(payload).await.0, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn full_page_editor_saves_reloads_and_opens_the_scheduled_details() {
    setup().await;
    let address = rahti::browser::serve(crate::routes::router()).await;
    let Some(mut browser) = rahti::browser::Browser::launch().await.unwrap() else {
        return;
    };
    authenticate_browser(&mut browser, address).await;
    browser
        .goto(&format!("http://{address}/calendar?new=1"))
        .await
        .unwrap();
    browser
        .wait_for("!!document.querySelector('.calendar-editor')")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.calendar-editor').hidden && document.activeElement.name === 'title'").await.unwrap();
    browser
        .type_text("Browser scheduled planning")
        .await
        .unwrap();
    browser.click("input[name=attendees]").await.unwrap();
    browser.type_text("alex@example.com, Avery").await.unwrap();
    browser.click("input[name=location]").await.unwrap();
    browser.type_text("Planning room").await.unwrap();
    browser.click("textarea[name=details]").await.unwrap();
    browser.type_text("Discuss next release").await.unwrap();
    let _: bool = browser
        .eval("document.activeElement.select(); true")
        .await
        .unwrap();
    browser.click("button[aria-label=Bold]").await.unwrap();
    let fits: bool=browser.eval("(() => {const editor=document.querySelector('.calendar-editor').getBoundingClientRect();return editor.width > innerWidth * .8 && editor.bottom <= innerHeight && document.querySelector('.calendar-editor-scroll').scrollWidth <= document.querySelector('.calendar-editor-scroll').clientWidth;})()").await.unwrap();
    assert!(
        fits,
        "the meeting editor must fill the app content and avoid horizontal overflow"
    );
    browser
        .click(".calendar-editor-header button:first-child")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.calendar-editor').hidden && !new URLSearchParams(location.search).has('new') && Array.from(document.querySelectorAll('.calendar-event')).some(el=>el.textContent.includes('Browser scheduled planning'))").await.unwrap();
    browser
        .goto(&format!("http://{address}/calendar"))
        .await
        .unwrap();
    browser.wait_for("Array.from(document.querySelectorAll('.calendar-event')).some(el=>el.textContent.includes('Browser scheduled planning'))").await.unwrap();
    browser
        .click("button[aria-label='Browser scheduled planning']")
        .await
        .unwrap();
    browser.wait_for("document.querySelector('.calendar-event-dialog').open && document.querySelector('.calendar-event-dialog').textContent.includes('Planning room') && document.querySelector('.calendar-event-dialog').textContent.includes('alex@example.com') && document.querySelector('.calendar-event-dialog').textContent.includes('Discuss next release')").await.unwrap();
    browser.wait_for("Array.from(document.querySelectorAll('.calendar-event-dialog strong')).some(el => !el.hidden && el.textContent === 'Discuss next release')").await.unwrap();
    browser.press("Escape").await.unwrap();
    // The dialog animates closed; the page behind it takes clicks once it has.
    browser
        .wait_for("!document.querySelector('.calendar-event-dialog').open")
        .await
        .unwrap();
    browser
        .click("main header button:has(svg.lucide-plus)")
        .await
        .unwrap();
    browser.wait_for("!document.querySelector('.calendar-editor').hidden && document.activeElement.name === 'title'").await.unwrap();
    browser
        .click(".calendar-editor-header button:nth-child(2)")
        .await
        .unwrap();
    browser
        .wait_for("document.querySelector('.calendar-editor').hidden")
        .await
        .unwrap();
    assert!(browser.errors().is_empty(), "{:?}", browser.errors());
}
