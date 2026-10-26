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
use crate::components::icon::Icon;
use crate::rahti::time::NaiveDate;
use crate::rahti::{Html, Metadata, Result, Validator, html, rpc};

pub fn metadata() -> Metadata {
    Metadata::new()
        .title("Calendar")
        .description("Your work week, meetings included.")
}

#[derive(Deserialize)]
pub struct CalendarQuery {
    new: Option<String>,
}

pub async fn page(Query(query): Query<CalendarQuery>) -> Html {
    let start_compose = query.new.as_deref() == Some("1");
    html! {
        <AppShell active="calendar">
            <main
                class="relative flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
            >
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
                        <button
                            type="button"
                            class="flex h-9 items-center gap-2 rounded-md border border-edge bg-raised px-4 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                        >
                            <Icon name="hash" class="h-4 w-4" />
                            "Join with an ID"
                        </button>
                        <a
                            href="/meet"
                            class="flex h-9 items-center gap-2 rounded-md border border-edge bg-raised px-4 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                        >
                            <Icon name="video" class="h-4 w-4" />
                            "Meet now"
                        </a>
                        <button
                            type="button"
                            class="flex h-9 items-center gap-2 rounded-md bg-brand px-4 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover"
                            onclick={openCompose()}
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
                            onclick={setOffset(0)}
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
                                    <span class="absolute -top-2 right-3 text-[11px] text-subtle">
                                        {hour.label}
                                    </span>
                                </div>
                            </template>
                        </div>
                        <div class="relative flex-1">
                            <div class={"grid " + gridColumns}>
                                <template pp-for="cell in cells">
                                    <div key={cell.key} class="h-14 border-b border-l border-line">
                                        <div class="h-1/2 border-b border-dotted border-line"></div>
                                    </div>
                                </template>
                            </div>
                            <template pp-for="meeting in placed">
                                <div
                                    key={meeting.id}
                                    class="absolute overflow-hidden rounded-md border-l-[3px] border-brand bg-brand-soft px-2 py-1"
                                    pp-style={meeting.style}
                                >
                                    <p class="truncate text-[11px] font-semibold text-ink">
                                        {meeting.title}
                                    </p>
                                    <p class="truncate text-[11px] text-muted">{meeting.label}</p>
                                </div>
                            </template>
                            <div
                                class="pointer-events-none absolute inset-x-0 z-10"
                                pp-style={nowStyle}
                                hidden={!showNow}
                            >
                                <div class="border-t border-dashed border-now"></div>
                                <div class="absolute -left-1 -top-1 h-2 w-2 rounded-full bg-now">
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div
                    class="absolute inset-0 z-30 flex items-center justify-center bg-black/60 px-6"
                    hidden={!composeOpen}
                >
                    <div
                        class="w-full max-w-[520px] rounded-lg border border-edge bg-raised px-9 py-8 shadow-2xl shadow-black/70"
                    >
                        <h2 class="text-[17px] font-semibold text-ink">"New meeting"</h2>
                        <form class="mt-6" onsubmit={save(event)} novalidate>
                            <label
                                class="block text-xs font-semibold text-muted"
                                for="meeting-title"
                            >
                                "Title"
                            </label>
                            <input
                                id="meeting-title"
                                name="title"
                                type="text"
                                class="mt-2 h-9 w-full rounded border border-edge bg-surface px-3 text-[13px] text-ink placeholder:text-muted focus:border-brand focus:outline-none"
                                placeholder="Weekly sync"
                                autocomplete="off"
                            />
                            <p class="mt-1.5 text-xs text-busy" hidden={!errors.title}>
                                {errors.title}
                            </p>
                            <div class="mt-5 grid grid-cols-3 gap-4">
                                <div>
                                    <label
                                        class="block text-xs font-semibold text-muted"
                                        for="meeting-date"
                                    >
                                        "Date"
                                    </label>
                                    <input
                                        id="meeting-date"
                                        name="date"
                                        type="date"
                                        class="mt-2 h-9 w-full rounded border border-edge bg-surface px-3 text-[13px] text-ink focus:border-brand focus:outline-none"
                                        defaultvalue={defaultDate}
                                    />
                                </div>
                                <div>
                                    <label
                                        class="block text-xs font-semibold text-muted"
                                        for="meeting-start"
                                    >
                                        "Start"
                                    </label>
                                    <input
                                        id="meeting-start"
                                        name="start"
                                        type="time"
                                        class="mt-2 h-9 w-full rounded border border-edge bg-surface px-3 text-[13px] text-ink focus:border-brand focus:outline-none"
                                        value="09:00"
                                    />
                                </div>
                                <div>
                                    <label
                                        class="block text-xs font-semibold text-muted"
                                        for="meeting-minutes"
                                    >
                                        "Length"
                                    </label>
                                    <select
                                        id="meeting-minutes"
                                        name="minutes"
                                        class="mt-2 h-9 w-full rounded border border-edge bg-surface px-2 text-[13px] text-ink focus:border-brand focus:outline-none"
                                    >
                                        <option value="30">"30 minutes"</option>
                                        <option value="60" selected>"1 hour"</option>
                                        <option value="90">"1.5 hours"</option>
                                        <option value="120">"2 hours"</option>
                                    </select>
                                </div>
                            </div>
                            <p class="mt-2 text-xs text-busy" hidden={!formError}>{formError}</p>
                            <div class="mt-8 flex items-center justify-end gap-3">
                                <button
                                    type="button"
                                    class="h-9 rounded border border-edge px-5 text-[13px] font-semibold text-ink transition-colors hover:bg-hover"
                                    onclick={closeCompose()}
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="submit"
                                    class="h-9 rounded bg-brand px-5 text-[13px] font-semibold text-white transition-colors hover:bg-brand-hover disabled:opacity-60"
                                    disabled={saving}
                                >
                                    {saving ? "Saving…" : "Save"}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
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
                    const [meetings, setMeetings] = pp.state([]);
                    const [composeOpen, setComposeOpen] = pp.state(@{start_compose});
                    const [saving, setSaving] = pp.state(false);
                    const [errors, setErrors] = pp.state({});
                    const [formError, setFormError] = pp.state("");

                    const scroller = pp.ref(null);
                    const now = new Date();

                    function pad(value) {
                        return value < 10 ? "0" + value : "" + value;
                    }

                    function dayKey(date) {
                        return date.getFullYear() + "-" + pad(date.getMonth() + 1) + "-" + pad(date.getDate());
                    }

                    const todayKey = dayKey(now);
                    const defaultDate = todayKey;

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
                            list.push(describe(new Date(now.getFullYear(), now.getMonth(), now.getDate() + offset)));
                            return list;
                        }

                        const weekday = now.getDay();
                        const shift = weekday === 0 ? -6 : 1 - weekday;
                        const first = new Date(now.getFullYear(), now.getMonth(), now.getDate() + shift + offset * 7);
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
                            " · " + length + "h";
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
                                id: meeting.id,
                                title: meeting.title,
                                label: clockLabel(meeting.start, meeting.length),
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

                    function closeCompose() {
                        setComposeOpen(false);
                    }

                    async function save(event) {
                        event.preventDefault();
                        if (saving) return;

                        const data = Object.fromEntries(new FormData(event.currentTarget));
                        setSaving(true);
                        setErrors({});
                        setFormError("");
                        try {
                            const meeting = await pp.rpc("create_meeting", data);
                            setMeetings((current) => current.concat([meeting]));
                            setComposeOpen(false);
                        } catch (failure) {
                            const fields = failure.errors || {};
                            setErrors({
                                title: fields.title ? fields.title[0] : "",
                            });
                            if (!fields.title) {
                                setFormError(failure.message || "The meeting could not be saved.");
                            }
                        } finally {
                            setSaving(false);
                        }
                    }
                "##</script>
            </main>
        </AppShell>
    }
}

/// A meeting placed on the grid.
#[derive(Serialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    /// The day the grid keys its columns by, `YYYY-MM-DD`.
    pub day: String,
    /// Start, in hours from midnight — `9.5` is half past nine.
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
