//! Calendar: a work-week grid the browser builds, and the meetings on it.
//!
//! The grid is browser work on purpose. Moving a week back or switching to
//! Week view changes nothing the server knows, and a round trip per arrow
//! would make the arrows feel like page loads. What the server does own is a
//! new meeting: its times are parsed and checked in Rust before the grid is
//! allowed to show it.

use axum::extract::Query;
use serde::{Deserialize, Serialize};

use crate::components::app_shell::AppShell;
use crate::components::calendar_meet_now::CalendarMeetNow;
use crate::components::icon::Icon;
use crate::components::join_meeting_by_id::JoinMeetingById;
use crate::rahti::time::NaiveDate;
use crate::rahti::{Html, Json, Metadata, Result, Validator, html, rpc};

#[rpc(auth)]
pub async fn schedule_meeting(
    input: crate::calendar::ScheduleInput,
) -> Result<Vec<crate::calendar::ScheduledMeeting>> {
    crate::calendar::schedule(input).await
}

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Calendar")
        .description("Your work week, meetings included.")
}

#[derive(Deserialize)]
pub struct CalendarQuery {
    new: Option<String>,
}

pub async fn page(Query(query): Query<CalendarQuery>) -> Result<Html> {
    let start_compose = query.new.as_deref() == Some("1");
    let saved_meetings = crate::calendar::list().await?;
    let app_now = crate::rahti::time::now();
    let app_date = app_now.format("%Y-%m-%d").to_string();
    let app_clock = app_now.format("%H:%M").to_string();
    let app_zone = crate::rahti::time::timezone().name();
    Ok(html! {
        <AppShell active="calendar">
            <main
                class="relative flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
            >
                <div class="contents" hidden={composeOpen}>
                    <header class="flex h-16 shrink-0 items-center justify-between px-6">
                        <div class="flex items-center gap-3">
                            <span
                                class="flex h-9 w-9 items-center justify-center rounded-md bg-brand text-white"
                            >
                                <Icon name="calendar" class="h-5 w-5" />
                            </span>
                            <h1 class="text-[20px] font-semibold tracking-tight text-ink">
                                "Calendar"
                            </h1>
                        </div>
                        <div class="flex items-center gap-2">
                            <JoinMeetingById />
                            <CalendarMeetNow />
                            <button
                                type="button"
                                class="flex h-9 items-center gap-2 rounded-md bg-brand px-4 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
                                onclick={openCompose()}
                                pp-ref={newMeetingButton}
                            >
                                <Icon name="plus" class="h-4 w-4" />
                                "New meeting"
                            </button>
                        </div>
                    </header>
                    <div class="flex h-12 shrink-0 items-center justify-between px-6">
                        <div class="flex items-center gap-1">
                            <button
                                type="button"
                                class="flex h-8 items-center gap-2 rounded-md px-3 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                onclick={goToday()}
                            >
                                <Icon name="calendar-day" class="h-4 w-4" />
                                "Today"
                            </button>
                            <button
                                type="button"
                                class="ml-1 flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Previous"
                                aria-label="Previous"
                                onclick={setOffset(value => value - 1)}
                            >
                                <Icon name="chevron-left" class="h-4 w-4" />
                            </button>
                            <button
                                type="button"
                                class="flex h-8 w-8 items-center justify-center rounded-md text-muted transition-colors hover:bg-hover hover:text-ink"
                                title="Next"
                                aria-label="Next"
                                onclick={setOffset(value => value + 1)}
                            >
                                <Icon name="chevron-right" class="h-4 w-4" />
                            </button>
                            <button
                                type="button"
                                class="ml-2 flex h-8 items-center gap-2 rounded-md px-2 text-[15px] font-semibold text-ink transition-colors hover:bg-hover"
                            >
                                {title}
                                <Icon name="chevron-down" class="h-4 w-4 text-muted" />
                            </button>
                        </div>
                        <div class="relative flex items-center gap-4">
                            <span class="text-[13px] text-muted">"You're up to date!"</span>
                            <button
                                type="button"
                                class="flex h-8 items-center gap-2 rounded-md border border-edge bg-raised px-3 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                aria-expanded={viewOpen}
                                onclick={setViewOpen(open => !open)}
                            >
                                <Icon name="calendar-day" class="h-4 w-4 text-muted" />
                                {view}
                                <Icon name="chevron-down" class="h-4 w-4 text-muted" />
                            </button>
                            <div
                                class="absolute right-0 top-9 z-20 w-40 rounded-md border border-edge bg-raised p-1 shadow-2xl shadow-black/60"
                                hidden={!viewOpen}
                            >
                                <button
                                    type="button"
                                    class="flex w-full items-center rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                                    onclick={pickView("Day")}
                                >
                                    "Day"
                                </button>
                                <button
                                    type="button"
                                    class="flex w-full items-center rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                                    onclick={pickView("Work week")}
                                >
                                    "Work week"
                                </button>
                                <button
                                    type="button"
                                    class="flex w-full items-center rounded px-3 py-2 text-left text-[13px] text-ink hover:bg-hover"
                                    onclick={pickView("Week")}
                                >
                                    "Week"
                                </button>
                            </div>
                        </div>
                    </div>
                    <div class="flex shrink-0 border-b border-line pr-2">
                        <div class="w-[72px] shrink-0"></div>
                        <div class={"grid flex-1 " + gridColumns}>
                            <template pp-for="day in days">
                                <div key={day.key} class={day.today ? HEAD_ON : HEAD_OFF}>
                                    <span class="text-[22px] font-semibold leading-none">
                                        {day.number}
                                    </span>
                                    <span class="mt-1.5 text-[12px] leading-none">{day.name}</span>
                                </div>
                            </template>
                        </div>
                    </div>
                    <div class="scroll-thin min-h-0 flex-1 overflow-y-auto" pp-ref={scroller}>
                        <div class="flex">
                            <div class="w-[72px] shrink-0">
                                <template pp-for="hour in hours">
                                    <div key={hour.key} class="relative h-14">
                                        <span
                                            class="absolute -top-2 right-3 text-[11px] text-subtle"
                                        >
                                            {hour.label}
                                        </span>
                                    </div>
                                </template>
                            </div>
                            <div class="relative flex-1">
                                <div class={"grid " + gridColumns}>
                                    <template pp-for="cell in cells">
                                        <div
                                            key={cell.key}
                                            class="h-14 border-b border-l border-line"
                                        >
                                            <div class="h-1/2 border-b border-dotted border-line">
                                            </div>
                                        </div>
                                    </template>
                                </div>
                                <template pp-for="meeting in placed">
                                    <button
                                        type="button"
                                        key={meeting.id}
                                        onclick={showEvent(meeting, event)}
                                        aria-label={meeting.title}
                                        class="calendar-event absolute overflow-hidden rounded-md border-l-[3px] border-brand bg-brand-soft px-2 py-1 text-left"
                                        pp-style={meeting.style}
                                    >
                                        <p class="truncate text-[11px] font-semibold text-ink">
                                            {meeting.title}
                                        </p>
                                        <p class="truncate text-[11px] text-muted">
                                            {meeting.label}
                                        </p>
                                    </button>
                                </template>
                                <div
                                    class="pointer-events-none absolute inset-x-0 z-10"
                                    pp-style={nowStyle}
                                    hidden={!showNow}
                                >
                                    <div class="border-t border-dashed border-now"></div>
                                    <div
                                        class="absolute -left-1 -top-1 h-2 w-2 rounded-full bg-now"
                                    >
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <section
                    class="calendar-editor"
                    hidden={!composeOpen}
                    onkeydown={editorKey(event)}
                    aria-label="New meeting editor"
                >
                    <header class="calendar-editor-header">
                        <span
                            class="flex h-9 w-9 items-center justify-center rounded-md bg-brand text-white"
                        >
                            <Icon name="calendar" class="h-5 w-5" />
                        </span>
                        <h2 class="text-[20px] font-semibold">"New meeting"</h2>
                        <span class="calendar-details-tab">"Details"</span>
                        <div class="ml-auto flex gap-2">
                            <button
                                type="button"
                                class="rounded bg-brand px-5 py-2 text-sm font-semibold text-white hover:bg-brand-hover disabled:opacity-50"
                                onclick={editorForm.current?.requestSubmit()}
                                disabled={saving}
                            >
                                {saving ? "Saving…" : "Save"}
                            </button>
                            <button
                                type="button"
                                class="rounded border border-edge px-5 py-2 text-sm font-semibold hover:bg-hover"
                                onclick={closeCompose()}
                                disabled={saving}
                            >
                                "Close"
                            </button>
                        </div>
                    </header>
                    <div class="calendar-editor-scroll scroll-thin">
                        <form
                            pp-ref={editorForm}
                            onsubmit={save(event)}
                            novalidate
                            class="calendar-editor-form"
                        >
                            <label class="calendar-timezone">
                                "Time zone: "
                                <select
                                    name="timezone"
                                    aria-label="Time zone"
                                    value={editorZone}
                                    onchange={setEditorZone(target.value)}
                                >
                                    <template pp-for="zone in timezones">
                                        <option key={zone} value={zone}>{zone}</option>
                                    </template>
                                </select>
                            </label>
                            <div class="calendar-field">
                                <Icon name="edit" class="h-5 w-5 text-muted" />
                                <input
                                    pp-ref={titleInput}
                                    name="title"
                                    aria-label="Meeting title"
                                    placeholder="Add title"
                                    maxlength="120"
                                    required
                                    autocomplete="off"
                                />
                            </div>
                            <p class="calendar-field-error" role="alert" hidden={!errors.title}>
                                {errors.title}
                            </p>
                            <div class="calendar-field">
                                <Icon name="person-add" class="h-5 w-5 text-muted" />
                                <input
                                    name="attendees"
                                    aria-label="Attendees"
                                    placeholder="Enter names or email addresses, separated by commas"
                                    maxlength="3000"
                                    autocomplete="off"
                                />
                            </div>
                            <p class="calendar-field-error" role="alert" hidden={!errors.attendees}>
                                {errors.attendees}
                            </p>
                            <div class="calendar-field">
                                <Icon name="clock" class="h-5 w-5 text-muted" />
                                <div class="calendar-datetime-row">
                                    <input
                                        name="start_date"
                                        type="date"
                                        aria-label="Start date"
                                        value={startDate}
                                        onchange={changeStartDate(target.value)}
                                        required
                                    />
                                    <input
                                        name="start_time"
                                        type="time"
                                        aria-label="Start time"
                                        value={startTime}
                                        onchange={changeStartTime(target.value)}
                                        disabled={allDay}
                                        required
                                    />
                                    <Icon name="arrow-right" class="h-5 w-5 shrink-0 text-muted" />
                                    <input
                                        name="end_date"
                                        type="date"
                                        aria-label="End date"
                                        value={endDate}
                                        onchange={setEndDate(target.value)}
                                        required
                                    />
                                    <input
                                        name="end_time"
                                        type="time"
                                        aria-label="End time"
                                        value={endTime}
                                        onchange={setEndTime(target.value)}
                                        disabled={allDay}
                                        required
                                    />
                                    <span class="text-xs text-muted" hidden={allDay}>
                                        {durationLabel}
                                    </span>
                                    <button
                                        type="button"
                                        role="switch"
                                        aria-label="All day"
                                        aria-checked={allDay}
                                        onclick={setAllDay(value => !value)}
                                        class="calendar-all-day-toggle"
                                    >
                                        <span></span>
                                    </button>
                                    <span class="text-sm text-muted">"All day"</span>
                                </div>
                            </div>
                            <div class="calendar-field">
                                <Icon name="repeat" class="h-5 w-5 text-muted" />
                                <div class="flex flex-wrap items-center gap-3">
                                    <select
                                        name="recurrence"
                                        aria-label="Repeat"
                                        value={recurrence}
                                        onchange={setRecurrence(target.value)}
                                    >
                                        <option value="none">"Does not repeat"</option>
                                        <option value="daily">"Daily"</option>
                                        <option value="weekly">"Weekly"</option>
                                        <option value="monthly">"Monthly"</option>
                                    </select>
                                    <label
                                        hidden={recurrence === "none"}
                                        class="flex items-center gap-2 text-sm text-muted"
                                    >
                                        "Occurrences"
                                        <input
                                            name="occurrences"
                                            type="number"
                                            min="1"
                                            max="52"
                                            value="10"
                                            aria-label="Occurrences"
                                        />
                                    </label>
                                </div>
                            </div>
                            <div class="calendar-field">
                                <Icon name="location" class="h-5 w-5 text-muted" />
                                <input
                                    name="location"
                                    aria-label="Location"
                                    placeholder="Add location"
                                    maxlength="300"
                                />
                            </div>
                            <div class="calendar-field calendar-description-row">
                                <Icon name="format" class="h-5 w-5 text-muted" />
                                <div class="min-w-0 flex-1">
                                    <div
                                        class="calendar-format-toolbar"
                                        role="toolbar"
                                        aria-label="Meeting details formatting"
                                    >
                                        <button
                                            type="button"
                                            aria-label="Bold"
                                            title="Bold"
                                            onclick={formatDetails("**", "**")}
                                        >
                                            <Icon name="bold" class="h-4 w-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Italic"
                                            title="Italic"
                                            onclick={formatDetails("_", "_")}
                                        >
                                            <Icon name="italic" class="h-4 w-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Bulleted list"
                                            title="Bulleted list"
                                            onclick={formatDetails("\n- ", "")}
                                        >
                                            <Icon name="list" class="h-4 w-4" />
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Numbered list"
                                            title="Numbered list"
                                            onclick={formatDetails("\n1. ", "")}
                                        >
                                            <Icon name="list-ordered" class="h-4 w-4" />
                                        </button>
                                    </div>
                                    <textarea
                                        pp-ref={detailsInput}
                                        name="details"
                                        aria-label="Meeting details"
                                        placeholder="Type details for this new meeting"
                                        maxlength="8000"
                                        rows="5"
                                    >
                                    </textarea>
                                    <p class="mt-2 text-xs text-muted">
                                        "Supports basic Markdown formatting."
                                    </p>
                                </div>
                            </div>
                            <p
                                class="calendar-save-error text-sm text-danger-ink"
                                role="alert"
                                hidden={!formError}
                            >
                                {formError}
                            </p>
                        </form>
                    </div>
                </section>
                <dialog
                    pp-ref={eventDialog}
                    class="calendar-event-dialog rounded-lg border border-edge bg-raised p-6 text-ink shadow-2xl"
                    aria-label="Meeting details"
                    onclose={eventTrigger.current?.focus()}
                >
                    <h2 class="text-lg font-semibold">{selectedEvent?.title || "Meeting"}</h2>
                    <p class="mt-3 text-sm text-muted">
                        {selectedEvent ? eventTime(selectedEvent) : ""}
                    </p>
                    <p class="mt-2 text-sm" hidden={!selectedEvent?.location}>
                        "Location: "
                        {selectedEvent?.location}
                    </p>
                    <p class="mt-2 text-sm" hidden={!selectedEvent?.attendees?.length}>
                        "Attendees: "
                        {selectedEvent?.attendees?.join(", ")}
                    </p>
                    <p
                        class="mt-2 text-sm"
                        hidden={!selectedEvent || selectedEvent.recurrence === "none"}
                    >
                        "Repeats "
                        {selectedEvent?.recurrence}
                    </p>
                    <p class="mt-4 whitespace-pre-wrap text-sm">
                        <template pp-for="part in detailParts">
                            <span key={part.key}>
                                <strong hidden={part.kind !== "bold"}>{part.text}</strong>
                                <em hidden={part.kind !== "italic"}>{part.text}</em>
                                <span hidden={part.kind !== "text"}>{part.text}</span>
                            </span>
                        </template>
                    </p>
                    <div class="mt-6 flex justify-end gap-3">
                        <button
                            type="button"
                            class="rounded border border-edge px-4 py-2 text-sm"
                            onclick={eventDialog.current?.close()}
                        >
                            "Close"
                        </button>
                        <a
                            class="rounded bg-brand px-4 py-2 text-sm font-semibold text-white"
                            href={selectedEvent ? "/meet/j/" + selectedEvent.event_id : "/calendar"}
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "Join meeting"
                        </a>
                    </div>
                </dialog>
                <style>r##"
                    .calendar-editor {display:flex;flex:1;flex-direction:column;min-height:0;}
                    .calendar-editor-header {display:flex;align-items:center;gap:14px;min-height:72px;padding:12px 28px;border-bottom:1px solid var(--color-edge);}
                    .calendar-details-tab {align-self:stretch;display:flex;align-items:center;padding:0 4px;margin-left:6px;border-bottom:3px solid var(--color-brand-ink);font-size:14px;font-weight:600;}
                    .calendar-editor-scroll {flex:1;overflow-y:auto;min-height:0;padding:18px 30px 36px;}
                    .calendar-editor-form {width:100%;max-width:1060px;}
                    .calendar-timezone {display:flex;align-items:center;gap:6px;color:var(--color-muted);font-size:13px;margin:0 0 32px;}
                    .calendar-timezone select {background:transparent;max-width:300px;}
                    .calendar-field {display:flex;align-items:center;gap:20px;margin:18px 0;}
                    .calendar-field > svg {flex-shrink:0;}
                    .calendar-field > input {flex:1;min-width:0;}
                    .calendar-field input,.calendar-field select,.calendar-field textarea {height:40px;border:0;border-radius:3px;background:var(--color-raised);padding:8px 12px;font-size:14px;color:var(--color-ink);outline:none;}
                    .calendar-field input:focus,.calendar-field select:focus,.calendar-field textarea:focus {box-shadow:inset 0 -2px var(--color-brand-ink);}
                    .calendar-field input::placeholder,.calendar-field textarea::placeholder {color:var(--color-muted);}
                    .calendar-datetime-row {display:flex;flex:1;flex-wrap:wrap;align-items:center;gap:8px;}
                    .calendar-datetime-row input[type=date] {width:150px;}
                    .calendar-datetime-row input[type=time] {width:112px;}
                    .calendar-field input[type=number] {width:72px;}
                    .calendar-datetime-row input:disabled {opacity:.4;}
                    .calendar-all-day-toggle {width:36px;height:20px;padding:3px;border-radius:12px;background:var(--color-hover);}
                    .calendar-all-day-toggle span {display:block;width:14px;height:14px;border-radius:50%;background:#ddd;transition:transform .15s;}
                    .calendar-all-day-toggle[aria-checked=true] {background:var(--color-brand);}
                    .calendar-all-day-toggle[aria-checked=true] span {transform:translateX(16px);}
                    .calendar-description-row {align-items:flex-start;}
                    .calendar-description-row > svg {margin-top:8px;}
                    .calendar-format-toolbar {display:flex;align-items:center;gap:5px;margin-bottom:4px;}
                    .calendar-format-toolbar button {padding:8px;border-radius:3px;color:var(--color-muted);}
                    .calendar-format-toolbar button:hover {background:var(--color-hover);color:var(--color-ink);}
                    .calendar-field textarea {display:block;width:100%;height:130px;resize:vertical;}
                    .calendar-field-error {margin-left:40px;color:var(--color-danger-ink);font-size:12px;}
                    .calendar-save-error {margin-left:40px;}
                    .calendar-event-dialog {margin:auto;width:min(480px,calc(100vw - 32px));max-height:calc(100dvh - 32px);overflow-y:auto;}
                    .calendar-event-dialog::backdrop {background:#0007;}
                    @media(max-width:700px) {.calendar-editor-header {padding:12px;gap:8px;flex-wrap:wrap;}.calendar-details-tab {display:none;}.calendar-editor-scroll {padding:12px;}.calendar-field {gap:10px;}.calendar-timezone {flex-wrap:wrap;}.calendar-editor-header h2 {font-size:17px;}}
                "##</style>
                <script>r##"
                    const HEAD_ON = "flex flex-col border-l border-l-line border-t-2 border-t-brand px-3 py-2.5 text-brand-ink";
                    const HEAD_OFF = "flex flex-col border-l border-l-line px-3 py-2.5 text-muted";
                    const HOUR_HEIGHT = 56;

                    const DAY_NAMES = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
                    const MONTHS = [
                        "January", "February", "March", "April", "May", "June",
                        "July", "August", "September", "October", "November", "December",
                    ];

                    const [offset, setOffset] = pp.state(0);
                    const [view, setView] = pp.state("Work week");
                    const [viewOpen, setViewOpen] = pp.state(false);
                    const [meetings, setMeetings] = pp.state(@{Json(&saved_meetings)});
                    const [composeOpen, setComposeOpen] = pp.state(@{start_compose});
                    const [saving, setSaving] = pp.state(false);
                    const [errors, setErrors] = pp.state({});
                    const [formError, setFormError] = pp.state("");
                    const editorForm = pp.ref(null);
                    const titleInput = pp.ref(null);
                    const detailsInput = pp.ref(null);
                    const newMeetingButton = pp.ref(null);
                    const eventDialog = pp.ref(null);
                    const eventTrigger = pp.ref(null);
                    const [selectedEvent, setSelectedEvent] = pp.state(null);
                    const detailParts = (selectedEvent?.details || '').split(/(\*\*[^*]+\*\*|_[^_]+_)/g).map((text,key) => {
                        if (text.startsWith('**') && text.endsWith('**')) return {key,kind:'bold',text:text.slice(2,-2)};
                        if (text.startsWith('_') && text.endsWith('_')) return {key,kind:'italic',text:text.slice(1,-1)};
                        return {key,kind:'text',text};
                    });
                    const [editorZone, setEditorZone] = pp.state(@{app_zone});
                    const timezones = Array.from(new Set([@{app_zone}, 'America/Managua', 'UTC', 'America/New_York', 'Europe/London', 'Asia/Tokyo']));
                    const [allDay, setAllDay] = pp.state(false);
                    const [recurrence, setRecurrence] = pp.state('none');
                    const initialStart = new Date(@{app_date} + 'T' + @{app_clock});
                    initialStart.setMinutes(Math.ceil((initialStart.getMinutes() + 1) / 30) * 30, 0, 0);
                    const initialEnd = new Date(initialStart.getTime() + 30 * 60000);
                    const [startDate, setStartDate] = pp.state(dayKey(initialStart));
                    const [startTime, setStartTime] = pp.state(clockValue(initialStart));
                    const [endDate, setEndDate] = pp.state(dayKey(initialEnd));
                    const [endTime, setEndTime] = pp.state(clockValue(initialEnd));
                    const [anchorDate, setAnchorDate] = pp.state(@{app_date});
                    const durationMinutes = (new Date(endDate + 'T' + endTime) - new Date(startDate + 'T' + startTime)) / 60000;
                    const durationLabel = Number.isFinite(durationMinutes) && durationMinutes > 0 ? (durationMinutes < 60 ? durationMinutes + 'm' : Math.round(durationMinutes / 6) / 10 + 'h') : '';

                    const scroller = pp.ref(null);
                    const now = new Date(@{app_date} + 'T' + @{app_clock});
                    const gridAnchor = new Date(anchorDate + 'T12:00');

                    function pad(value) {
                        return value < 10 ? "0" + value : "" + value;
                    }

                    function dayKey(date) {
                        return date.getFullYear() + "-" + pad(date.getMonth() + 1) + "-" + pad(date.getDate());
                    }

                    const todayKey = dayKey(now);
                    function clockValue(date) {return pad(date.getHours()) + ':' + pad(date.getMinutes());}
                    function changeStartDate(value) {
                        if (endDate === startDate || endDate < value) setEndDate(value);
                        setStartDate(value);
                    }
                    function changeStartTime(value) {
                        setStartTime(value);
                        const end = new Date(startDate + 'T' + value);
                        if (!Number.isFinite(end.getTime())) return;
                        end.setMinutes(end.getMinutes() + 30);
                        setEndDate(dayKey(end)); setEndTime(clockValue(end));
                    }
                    function formatDetails(before, after) {
                        const input = detailsInput.current;
                        if (!input) return;
                        const start = input.selectionStart; const end = input.selectionEnd;
                        const selection = input.value.slice(start,end) || 'text';
                        input.setRangeText(before + selection + after, start, end, 'select');
                        input.focus();
                    }
                    function editorKey(event) {if (event.key === 'Escape' && !saving) {event.preventDefault(); closeCompose();}}
                    function eventTime(meeting) {
                        const format = new Intl.DateTimeFormat(undefined, {dateStyle:'medium', timeStyle:'short', timeZone:meeting.timezone});
                        return format.format(new Date(meeting.starts_at)) + ' – ' + format.format(new Date(meeting.ends_at)) + ' (' + meeting.timezone + ')';
                    }
                    function showEvent(meeting, event) {eventTrigger.current = event.currentTarget; setSelectedEvent(meeting); eventDialog.current.showModal();}
                    function goToday() {setAnchorDate(@{app_date}); setOffset(0);}

                    function describe(date) {
                        const key = dayKey(date);
                        return {
                            key: key,
                            number: date.getDate(),
                            name: DAY_NAMES[date.getDay()],
                            month: date.getMonth(),
                            year: date.getFullYear(),
                            today: key === todayKey,
                        };
                    }

                    function buildDays() {
                        const list = [];
                        if (view === "Day") {
                            list.push(describe(new Date(gridAnchor.getFullYear(), gridAnchor.getMonth(), gridAnchor.getDate() + offset)));
                            return list;
                        }

                        const weekday = gridAnchor.getDay();
                        const shift = weekday === 0 ? -6 : 1 - weekday;
                        const first = new Date(gridAnchor.getFullYear(), gridAnchor.getMonth(), gridAnchor.getDate() + shift + offset * 7);
                        const count = view === "Week" ? 7 : 5;
                        if (view === "Week") {
                            first.setDate(first.getDate() - 1);
                        }
                        for (let index = 0; index < count; index += 1) {
                            list.push(describe(new Date(first.getFullYear(), first.getMonth(), first.getDate() + index)));
                        }
                        return list;
                    }

                    const days = buildDays();
                    const title = MONTHS[days[0].month] + " " + days[0].year;
                    // One column per day shown: Day, Work week or Week.
                    const GRID_COLUMNS = { 1: "grid-cols-1", 5: "grid-cols-5", 7: "grid-cols-7" };
                    const gridColumns = GRID_COLUMNS[days.length];

                    function hourLabel(hour) {
                        const suffix = hour < 12 ? " AM" : " PM";
                        const shown = hour % 12 === 0 ? 12 : hour % 12;
                        return shown + suffix;
                    }

                    const hours = [];
                    for (let hour = 0; hour < 24; hour += 1) {
                        hours.push({ key: hour, label: hourLabel(hour) });
                    }

                    const cells = [];
                    for (let hour = 0; hour < 24; hour += 1) {
                        for (let index = 0; index < days.length; index += 1) {
                            cells.push({ key: days[index].key + "@" + hour });
                        }
                    }

                    const minutesNow = now.getHours() * 60 + now.getMinutes();
                    const nowStyle = "top: " + ((minutesNow / 60) * HOUR_HEIGHT) + "px";
                    const showNow = days.some((day) => day.today);

                    function clockLabel(start, length) {
                        const startHour = Math.floor(start);
                        const startMinute = Math.round((start - startHour) * 60);
                        return hourLabel(startHour).replace(" ", ":" + pad(startMinute) + " ") +
                            " Â· " + length + "h";
                    }

                    const placed = meetings
                        .map((meeting) => {
                            const column = days.findIndex((day) => day.key === meeting.day);
                            if (column < 0) return null;
                            const width = 100 / days.length;
                            const style = "top: " + (meeting.start * HOUR_HEIGHT) + "px;" +
                                " height: " + Math.max(22, meeting.length * HOUR_HEIGHT - 4) + "px;" +
                                " left: calc(" + (column * width) + "% + 3px);" +
                                " width: calc(" + width + "% - 6px);";
                            return {
                                ...meeting,
                                id: meeting.id,
                                title: meeting.title,
                                label: meeting.all_day ? 'All day' : clockLabel(meeting.start, meeting.length),
                                style: style,
                            };
                        })
                        .filter((meeting) => meeting !== null);

                    pp.effect(() => {
                        const node = scroller.current;
                        if (!node) return;
                        node.scrollTop = Math.max(0, (now.getHours() - 2) * HOUR_HEIGHT);
                    }, []);

                    function pickView(value) {
                        setView(value);
                        setViewOpen(false);
                        setOffset(0);
                    }

                    function openCompose() {
                        setErrors({});
                        setFormError("");
                        setComposeOpen(true);
                    }
                    const editorWasOpen = pp.ref(false);
                    const focusEditor = pp.event(() => (composeOpen ? titleInput : newMeetingButton).current?.focus());
                    pp.effect(() => {
                        if (!composeOpen && !editorWasOpen.current) return;
                        editorWasOpen.current = composeOpen;
                        const frame = requestAnimationFrame(focusEditor);
                        return () => cancelAnimationFrame(frame);
                    }, [composeOpen, focusEditor]);

                    function closeCompose() {
                        if (saving) return;
                        setComposeOpen(false);
                        clearComposeUrl();
                        newMeetingButton.current?.focus();
                    }
                    function clearComposeUrl() {
                        const url = new URL(location.href);
                        if (!url.searchParams.has('new')) return;
                        url.searchParams.delete('new');
                        history.replaceState(history.state, '', url.pathname + url.search + url.hash);
                    }

                    async function save(event) {
                        event.preventDefault();
                        if (saving) return;

                        const data = Object.fromEntries(new FormData(event.currentTarget));
                        setSaving(true);
                        setErrors({});
                        setFormError("");
                        try {
                            const created = await pp.rpc("schedule_meeting", {input:{...data, all_day:allDay, occurrences:recurrence === 'none' ? 1 : Number(data.occurrences), start_time:data.start_time || '00:00', end_time:data.end_time || '00:00'}});
                            setMeetings((current) => current.concat(created));
                            if (created.length) {setAnchorDate(created[0].day); setOffset(0); setView('Week');}
                            setComposeOpen(false);
                            clearComposeUrl();
                            editorForm.current?.reset(); setAllDay(false); setRecurrence('none');
                            setStartDate(dayKey(initialStart)); setStartTime(clockValue(initialStart));
                            setEndDate(dayKey(initialEnd)); setEndTime(clockValue(initialEnd));
                            newMeetingButton.current?.focus();
                        } catch (failure) {
                            const fields = failure.errors || {};
                            setErrors({
                                title: fields.title ? fields.title[0] : "",
                                attendees: fields.attendees ? fields.attendees[0] : "",
                            });
                            const first = Object.values(fields).flat()[0];
                            setFormError(first || failure.message || "The meeting could not be saved.");
                        } finally {
                            setSaving(false);
                        }
                    }
                "##</script>
            </main>
        </AppShell>
    })
}

/// A meeting placed on the grid.
#[derive(Serialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    /// The day the grid keys its columns by, `YYYY-MM-DD`.
    pub day: String,
    /// Start, in hours from midnight â€” `9.5` is half past nine.
    pub start: f64,
    /// Length, in hours.
    pub length: f64,
}

/// Put a meeting on the calendar.
///
/// Every field arrives as the string a form sends, so the parsing is the
/// check: a date the calendar does not have, a time outside the day, or a
/// length nobody offered comes back as a 422 naming the field.
#[rpc(auth)]
pub async fn create_meeting(
    title: String,
    date: String,
    start: String,
    minutes: String,
) -> Result<Meeting> {
    let title = title.trim().to_string();
    let day = NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").ok();
    let clock = parse_clock(start.trim());
    let length = minutes.trim().parse::<u32>().ok();

    let mut validator = Validator::new();
    validator
        .required("title", &title)
        .max_chars("title", &title, 120)
        .check("date", day.is_some(), "Choose a date.")
        .check("start", clock.is_some(), "Choose a start time.")
        .check(
            "minutes",
            matches!(length, Some(30) | Some(60) | Some(90) | Some(120)),
            "Choose a length.",
        );
    validator.finish()?;

    let day = day.expect("the date was checked");
    let clock = clock.expect("the start time was checked");
    let length = length.expect("the length was checked");

    Ok(Meeting {
        id: format!("{}-{}", day.format("%Y%m%d"), (clock * 60.0) as i64),
        title,
        day: day.format("%Y-%m-%d").to_string(),
        start: clock,
        length: f64::from(length) / 60.0,
    })
}

/// `"14:30"` as hours from midnight, or `None` when it is not a time of day.
fn parse_clock(value: &str) -> Option<f64> {
    let (hours, minutes) = value.split_once(':')?;
    let hours: u32 = hours.parse().ok()?;
    let minutes: u32 = minutes.parse().ok()?;

    if hours > 23 || minutes > 59 {
        return None;
    }

    Some(f64::from(hours) + f64::from(minutes) / 60.0)
}
