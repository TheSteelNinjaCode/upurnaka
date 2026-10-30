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
use crate::components::aria::{bound, hidden, labelled, named, plain};
use crate::components::calendar_meet_now::CalendarMeetNow;
use crate::components::icon::Icon;
use crate::components::join_meeting_by_id::JoinMeetingById;
use crate::components::rahti_ui::attrs::{Attrs, ui_expr};
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};
use crate::components::rahti_ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use crate::components::rahti_ui::input::Input;
use crate::components::rahti_ui::label::Label;
use crate::components::rahti_ui::popover::{Popover, PopoverContent, PopoverTrigger};
use crate::components::rahti_ui::select::{Select, SelectItem};
use crate::components::rahti_ui::switch::Switch;
use crate::components::rahti_ui::textarea::Textarea;
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

/// One row of the meeting editor: an icon, then the field.
const ROW: &str = "my-[18px] flex items-center gap-5 max-[700px]:gap-2.5";

/// A field that takes the rest of its row.
const FIELD: &str = "h-10! min-w-0 flex-1";

/// A field's message, lined up under the field rather than its icon.
const FIELD_ERROR: &str = "ml-10 text-xs text-danger-ink";

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
    let view_trigger = button_variants(
        ButtonVariant::Outline,
        ButtonSize::Sm,
        "text-[13px] font-semibold",
    );
    let close_event = button_variants(ButtonVariant::Outline, ButtonSize::Default, "");
    let join_event = button_variants(ButtonVariant::Default, ButtonSize::Default, "font-semibold");
    let date_trigger = button_variants(
        ButtonVariant::Ghost,
        ButtonSize::Sm,
        "calendar-date-trigger ml-2 text-[15px] font-semibold aria-expanded:text-brand-ink",
    );
    Ok(html! {
        <AppShell active="calendar">
            <main
                class="relative flex min-h-0 flex-1 flex-col rounded-tl-xl border-l border-t border-line bg-surface"
            >
                <div class="contents" hidden={composeOpen}>
                    <header class="flex h-16 shrink-0 items-center justify-between px-6">
                        <div class="flex items-center gap-3">
                            <span
                                class="flex size-9 items-center justify-center rounded-md bg-brand text-white"
                            >
                                <Icon name="calendar" class="size-5" />
                            </span>
                            <h1 class="text-[20px] font-semibold tracking-tight text-ink">
                                "Calendar"
                            </h1>
                        </div>
                        <div class="flex items-center gap-2">
                            <JoinMeetingById />
                            <CalendarMeetNow />
                            <Button class="text-[13px] font-semibold" onclick={openCompose()}>
                                <span class="contents" pp-ref={newMeetingMarker}>
                                    <Icon name="plus" class="size-4" />
                                </span>
                                "New meeting"
                            </Button>
                        </div>
                    </header>
                    <div class="flex h-12 shrink-0 items-center justify-between px-6">
                        <div class="flex items-center gap-1">
                            <Button
                                variant="ghost"
                                size="sm"
                                class="text-[13px] font-semibold"
                                onclick={goToday()}
                            >
                                <Icon name="calendar-day" class="size-4" />
                                "Today"
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="ml-1 text-muted-foreground"
                                attrs=@{labelled("Previous")}
                                onclick={shiftRange(-1)}
                            >
                                <Icon name="chevron-left" class="size-4" />
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon-sm"
                                class="text-muted-foreground"
                                attrs=@{labelled("Next")}
                                onclick={shiftRange(1)}
                            >
                                <Icon name="chevron-right" class="size-4" />
                            </Button>
                            <Popover>
                                <PopoverTrigger
                                    name="calendar-date"
                                    class=@{date_trigger.as_str()}
                                    attrs=@{named("Choose calendar date")}
                                >
                                    {title}
                                    <Icon name="chevron-down" class="size-4" />
                                </PopoverTrigger>
                                <PopoverContent
                                    name="calendar-date"
                                    align="start"
                                    class="calendar-date-picker w-[304px] max-w-[calc(100vw-32px)] rounded-[5px] border border-line bg-popover p-[18px] text-ink shadow-xl shadow-black/30"
                                    ontoggle={toggleDatePicker(event.newState)}
                                    attrs=@{named("Choose calendar date")}
                                >
                                    <div
                                        class="calendar-picker-header mb-3.5 flex items-center gap-2.5"
                                    >
                                        <h2 class="flex-1 text-base font-bold" aria-live="polite">
                                            {pickerTitle}
                                        </h2>
                                        <button
                                            type="button"
                                            class="grid size-[30px] place-items-center rounded-[3px] text-ink hover:bg-hover"
                                            aria-label="Previous month"
                                            onclick={movePickerMonth(-1)}
                                        >
                                            <Icon name="chevron-left" class="size-5" />
                                        </button>
                                        <button
                                            type="button"
                                            class="grid size-[30px] place-items-center rounded-[3px] text-ink hover:bg-hover"
                                            aria-label="Next month"
                                            onclick={movePickerMonth(1)}
                                        >
                                            <Icon name="chevron-right" class="size-5" />
                                        </button>
                                    </div>
                                    <div class="calendar-picker-grid grid grid-cols-7">
                                        <template pp-for="weekday in PICKER_WEEKDAYS">
                                            <span
                                                key={weekday.name}
                                                class="calendar-picker-weekday grid h-8 place-items-center text-sm"
                                                title={weekday.name}
                                            >
                                                {weekday.short}
                                            </span>
                                        </template>
                                        <template pp-for="date in pickerDates">
                                            <button
                                                key={date.key}
                                                type="button"
                                                value={date.key}
                                                class={date.classes}
                                                aria-label={date.label}
                                                aria-pressed={date.selected}
                                                aria-current={date.today ? "date" : "false"}
                                                tabindex={date.key === pickerTabDate ? "0" : "-1"}
                                                pp-ref={element => pickerElements.current[date.key] = element}
                                                onclick={chooseCalendarDate(date.key)}
                                                onfocus={setPickerFocus(date.key)}
                                                onkeydown={pickerKey(event, date.key)}
                                            >
                                                <span>{date.number}</span>
                                            </button>
                                        </template>
                                    </div>
                                    <button
                                        type="button"
                                        popovertarget="popover-calendar-date"
                                        popovertargetaction="hide"
                                        pp-ref={pickerCloser}
                                        hidden
                                    >
                                    </button>
                                </PopoverContent>
                            </Popover>
                        </div>
                        <div class="flex items-center gap-4">
                            <DropdownMenu>
                                <DropdownMenuTrigger
                                    menu="calendar-view"
                                    class=@{view_trigger.as_str()}
                                    attrs=@{named("Calendar view")}
                                >
                                    <Icon
                                        name="calendar-day"
                                        class="size-4 text-muted-foreground"
                                    />
                                    {view}
                                    <Icon
                                        name="chevron-down"
                                        class="size-4 text-muted-foreground"
                                    />
                                </DropdownMenuTrigger>
                                <DropdownMenuContent menu="calendar-view" align="end" class="w-40">
                                    <template pp-for="choice in VIEWS">
                                        <DropdownMenuItem
                                            key={choice}
                                            class="text-[13px]"
                                            onclick={pickView(choice)}
                                        >
                                            {choice}
                                            <span class="ml-auto" hidden={view !== choice}>
                                                <Icon name="check" class="size-4 text-brand-ink" />
                                            </span>
                                        </DropdownMenuItem>
                                    </template>
                                </DropdownMenuContent>
                            </DropdownMenu>
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
                                        <button
                                            key={cell.key}
                                            type="button"
                                            value={cell.key}
                                            class={slotClass(cell)}
                                            tabindex={selectedSlot === cell.key ? "0" : "-1"}
                                            aria-label={cell.label}
                                            aria-pressed={selectedSlot === cell.key}
                                            pp-ref={element => slotElements.current[cell.key] = element}
                                            onfocus={setSelection(cell.key)}
                                            onclick={openCompose(cell)}
                                            onkeydown={slotKey(event, cell)}
                                        >
                                        </button>
                                    </template>
                                </div>
                                <template pp-for="meeting in placed">
                                    <div key={meeting.id} class="absolute" pp-style={meeting.style}>
                                        <Button
                                            variant="ghost"
                                            class="calendar-event size-full! flex-col! items-start! justify-start! gap-0 overflow-hidden rounded-md border-l-[3px] border-brand bg-brand-soft px-2! py-1 text-left hover:bg-brand-soft/80"
                                            attrs=@{bound("aria-label", ui_expr!(aria-label={meeting.title}))}
                                            value={meeting.id}
                                            onclick={showEvent(event.currentTarget.value)}
                                        >
                                            <span
                                                class="w-full truncate text-[11px] font-semibold text-ink"
                                            >
                                                {meeting.title}
                                            </span>
                                            <span
                                                class="w-full truncate text-[11px] font-normal text-muted-foreground"
                                            >
                                                {meeting.label}
                                            </span>
                                        </Button>
                                    </div>
                                </template>
                                <div
                                    class="calendar-now pointer-events-none absolute inset-x-0 z-10"
                                    pp-style={nowStyle}
                                    hidden={!showNow}
                                >
                                    <div
                                        class="calendar-now-guide absolute left-0 border-t border-dashed border-now"
                                        pp-style={nowGuideStyle}
                                    >
                                    </div>
                                    <div
                                        class="calendar-now-today absolute border-t-2 border-now"
                                        pp-style={nowTodayStyle}
                                    >
                                        <span
                                            class="calendar-now-dot absolute -left-0.5 -top-[3px] size-1 rounded-full bg-now"
                                        >
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <section
                    class="calendar-editor flex min-h-0 flex-1 flex-col"
                    hidden={!composeOpen}
                    onkeydown={editorKey(event)}
                    aria-label="New meeting editor"
                >
                    <header
                        class="calendar-editor-header flex min-h-[72px] items-center gap-3.5 border-b border-edge px-7 py-3 max-[700px]:flex-wrap max-[700px]:gap-2 max-[700px]:p-3"
                    >
                        <span
                            class="flex size-9 items-center justify-center rounded-md bg-brand text-white"
                        >
                            <Icon name="calendar" class="size-5" />
                        </span>
                        <h2 class="text-[20px] font-semibold max-[700px]:text-[17px]">
                            "New meeting"
                        </h2>
                        <span
                            class="ml-1.5 flex items-center self-stretch border-b-[3px] border-brand-ink px-1 text-sm font-semibold max-[700px]:hidden"
                        >
                            "Details"
                        </span>
                        <div class="ml-auto flex gap-2">
                            <Button
                                class="px-5 font-semibold"
                                onclick={editorForm.current?.requestSubmit()}
                                disabled={saving}
                            >
                                {saving ? "Saving…" : "Save"}
                            </Button>
                            <Button
                                variant="outline"
                                class="px-5 font-semibold"
                                onclick={closeCompose()}
                                disabled={saving}
                            >
                                "Close"
                            </Button>
                        </div>
                    </header>
                    <div
                        class="calendar-editor-scroll scroll-thin min-h-0 flex-1 overflow-y-auto px-[30px] pt-[18px] pb-9 max-[700px]:p-3"
                    >
                        <form
                            pp-ref={editorForm}
                            onsubmit={save(event)}
                            novalidate
                            class="w-full max-w-[1060px]"
                        >
                            <div
                                class="mb-8 flex items-center gap-1.5 text-[13px] text-muted-foreground max-[700px]:flex-wrap"
                            >
                                <Label r#for="calendar-timezone" class="font-normal">
                                    "Time zone:"
                                </Label>
                                <Select
                                    id="calendar-timezone"
                                    name="timezone"
                                    size="sm"
                                    class="max-w-[300px] border-transparent shadow-none"
                                    value={editorZone}
                                    onchange={setEditorZone(target.value)}
                                >
                                    <template pp-for="zone in timezones">
                                        <SelectItem key={zone} value={zone}>{zone}</SelectItem>
                                    </template>
                                </Select>
                            </div>
                            <div class=@{ROW}>
                                <Icon name="edit" class="size-5 shrink-0 text-muted-foreground" />
                                <Input
                                    name="title"
                                    placeholder="Add title"
                                    required
                                    class=@{FIELD}
                                    attrs=@{named("Meeting title")
                                        .set("maxlength", "120")
                                        .set("autocomplete", "off")}
                                />
                            </div>
                            <p class=@{FIELD_ERROR} role="alert" hidden={!errors.title}>
                                {errors.title}
                            </p>
                            <div class=@{ROW}>
                                <Icon
                                    name="person-add"
                                    class="size-5 shrink-0 text-muted-foreground"
                                />
                                <Input
                                    name="attendees"
                                    placeholder="Enter names or email addresses, separated by commas"
                                    class=@{FIELD}
                                    attrs=@{named("Attendees")
                                        .set("maxlength", "3000")
                                        .set("autocomplete", "off")}
                                />
                            </div>
                            <p class=@{FIELD_ERROR} role="alert" hidden={!errors.attendees}>
                                {errors.attendees}
                            </p>
                            <div class=@{ROW}>
                                <Icon name="clock" class="size-5 shrink-0 text-muted-foreground" />
                                <div class="flex flex-1 flex-wrap items-center gap-2">
                                    <Input
                                        name="start_date"
                                        r#type="date"
                                        required
                                        class="h-10! w-[150px]"
                                        attrs=@{named("Start date")}
                                        value={startDate}
                                        onchange={changeStartDate(target.value)}
                                    />
                                    <Input
                                        name="start_time"
                                        r#type="time"
                                        required
                                        class="h-10! w-[112px]"
                                        attrs=@{named("Start time")}
                                        value={startTime}
                                        onchange={changeStartTime(target.value)}
                                        disabled={allDay}
                                    />
                                    <Icon
                                        name="arrow-right"
                                        class="size-5 shrink-0 text-muted-foreground"
                                    />
                                    <Input
                                        name="end_date"
                                        r#type="date"
                                        required
                                        class="h-10! w-[150px]"
                                        attrs=@{named("End date")}
                                        value={endDate}
                                        onchange={setEndDate(target.value)}
                                    />
                                    <Input
                                        name="end_time"
                                        r#type="time"
                                        required
                                        class="h-10! w-[112px]"
                                        attrs=@{named("End time")}
                                        value={endTime}
                                        onchange={setEndTime(target.value)}
                                        disabled={allDay}
                                    />
                                    <span class="text-xs text-muted-foreground" hidden={allDay}>
                                        {durationLabel}
                                    </span>
                                    <Switch
                                        id="calendar-all-day"
                                        class="calendar-all-day-toggle"
                                        attrs=@{named("All day")}
                                        checked={allDay}
                                        onchange={setAllDay(target.checked)}
                                    />
                                    <Label
                                        r#for="calendar-all-day"
                                        class="font-normal text-muted-foreground"
                                    >
                                        "All day"
                                    </Label>
                                </div>
                            </div>
                            <div class=@{ROW}>
                                <Icon name="repeat" class="size-5 shrink-0 text-muted-foreground" />
                                <div class="flex flex-wrap items-center gap-3">
                                    <Select
                                        name="recurrence"
                                        class="h-10"
                                        attrs=@{named("Repeat")}
                                        value={recurrence}
                                        onchange={setRecurrence(target.value)}
                                    >
                                        <SelectItem value="none" selected>
                                            "Does not repeat"
                                        </SelectItem>
                                        <SelectItem value="daily">"Daily"</SelectItem>
                                        <SelectItem value="weekly">"Weekly"</SelectItem>
                                        <SelectItem value="monthly">"Monthly"</SelectItem>
                                    </Select>
                                    <div
                                        hidden={recurrence === "none"}
                                        class="flex items-center gap-2"
                                    >
                                        <Label
                                            r#for="calendar-occurrences"
                                            class="font-normal text-muted-foreground"
                                        >
                                            "Occurrences"
                                        </Label>
                                        <Input
                                            id="calendar-occurrences"
                                            name="occurrences"
                                            r#type="number"
                                            value="10"
                                            class="h-10! w-[72px]"
                                            attrs=@{Attrs::new().set("min", "1").set("max", "52")}
                                        />
                                    </div>
                                </div>
                            </div>
                            <div class=@{ROW}>
                                <Icon
                                    name="location"
                                    class="size-5 shrink-0 text-muted-foreground"
                                />
                                <Input
                                    name="location"
                                    placeholder="Add location"
                                    class=@{FIELD}
                                    attrs=@{named("Location").set("maxlength", "300")}
                                />
                            </div>
                            <div class="my-[18px] flex items-start gap-5 max-[700px]:gap-2.5">
                                <Icon
                                    name="format"
                                    class="mt-2 size-5 shrink-0 text-muted-foreground"
                                />
                                <div class="min-w-0 flex-1">
                                    <div
                                        class="mb-1 flex items-center gap-1"
                                        role="toolbar"
                                        aria-label="Meeting details formatting"
                                    >
                                        <Button
                                            variant="ghost"
                                            size="icon-sm"
                                            class="text-muted-foreground"
                                            attrs=@{(labelled("Bold")).merge(&plain())}
                                            onclick={formatDetails("**", "**")}
                                        >
                                            <Icon name="bold" class="size-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon-sm"
                                            class="text-muted-foreground"
                                            attrs=@{(labelled("Italic")).merge(&plain())}
                                            onclick={formatDetails("_", "_")}
                                        >
                                            <Icon name="italic" class="size-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon-sm"
                                            class="text-muted-foreground"
                                            attrs=@{(labelled("Bulleted list")).merge(&plain())}
                                            onclick={formatDetails("\n- ", "")}
                                        >
                                            <Icon name="list" class="size-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon-sm"
                                            class="text-muted-foreground"
                                            attrs=@{(labelled("Numbered list")).merge(&plain())}
                                            onclick={formatDetails("\n1. ", "")}
                                        >
                                            <Icon name="list-ordered" class="size-4" />
                                        </Button>
                                    </div>
                                    <Textarea
                                        name="details"
                                        placeholder="Type details for this new meeting"
                                        rows="5"
                                        class="h-[130px] resize-y field-sizing-fixed!"
                                        attrs=@{named("Meeting details").set("maxlength", "8000")}
                                    />
                                    <p class="mt-2 text-xs text-muted-foreground">
                                        "Supports basic Markdown formatting."
                                    </p>
                                </div>
                            </div>
                            <p
                                class="calendar-save-error ml-10 text-sm text-danger-ink"
                                role="alert"
                                hidden={!formError}
                            >
                                {formError}
                            </p>
                        </form>
                    </div>
                </section>
                <Dialog>
                    <DialogTrigger attrs=@{hidden()}>
                        <span pp-ref={eventOpener}></span>
                    </DialogTrigger>
                    <DialogContent
                        class="calendar-event-dialog max-h-[calc(100dvh-32px)] overflow-y-auto sm:max-w-[480px]!"
                        attrs=@{named("Meeting details")}
                    >
                        <DialogHeader>
                            <DialogTitle>{selectedEvent?.title || "Meeting"}</DialogTitle>
                            <DialogDescription>
                                {selectedEvent ? eventTime(selectedEvent) : ""}
                            </DialogDescription>
                        </DialogHeader>
                        <div class="space-y-2 text-sm">
                            <p hidden={!selectedEvent?.location}>
                                "Location: "
                                {selectedEvent?.location}
                            </p>
                            <p hidden={!selectedEvent?.attendees?.length}>
                                "Attendees: "
                                {selectedEvent?.attendees?.join(", ")}
                            </p>
                            <p hidden={!selectedEvent || selectedEvent.recurrence === "none"}>
                                "Repeats "
                                {selectedEvent?.recurrence}
                            </p>
                            <p class="whitespace-pre-wrap pt-2">
                                <template pp-for="part in detailParts">
                                    <span key={part.key}>
                                        <strong hidden={part.kind !== "bold"}>{part.text}</strong>
                                        <em hidden={part.kind !== "italic"}>{part.text}</em>
                                        <span hidden={part.kind !== "text"}>{part.text}</span>
                                    </span>
                                </template>
                            </p>
                        </div>
                        <DialogFooter>
                            <DialogClose class=@{close_event.as_str()}>"Close"</DialogClose>
                            <a
                                class=@{join_event.as_str()}
                                href={selectedEvent ? "/meet/j/" + selectedEvent.event_id : "/calendar"}
                                target="_blank"
                                rel="noopener noreferrer"
                            >
                                "Join meeting"
                            </a>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
                <script>r##"
                    const PICKER_DAY = 'calendar-picker-day grid h-[38px] min-w-0 place-items-center text-sm focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ink [&>span]:grid [&>span]:size-[34px] [&>span]:place-items-center [&>span]:rounded-full';
                    function pickerDayClass(outside, inRange, selected) {
                        return PICKER_DAY
                            + (outside ? ' calendar-picker-outside' : '')
                            + (inRange ? ' calendar-picker-range bg-brand-soft text-ink' : outside ? ' text-muted-foreground' : '')
                            + (selected
                                ? ' calendar-picker-selected [&>span]:bg-brand [&>span]:text-brand-foreground'
                                : ' hover:[&>span]:bg-hover aria-[current=date]:[&>span]:text-brand-ink aria-[current=date]:[&>span]:shadow-[inset_0_0_0_1px_var(--color-brand-ink)]');
                    }
                    const SLOT = 'calendar-grid-slot relative h-7 min-w-0 cursor-default border-b border-l border-line p-0 hover:bg-hover focus-visible:z-1 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ink';
                    function slotClass(cell) {
                        return SLOT
                            + (cell.half % 2 ? ' calendar-hour-end' : ' [border-bottom-style:dotted]')
                            + (selectedSlot === cell.key ? ' calendar-slot-selected z-1 rounded-[2px] outline-2 -outline-offset-2 outline-ink' : '');
                    }
                    const HEAD_ON = "flex flex-col border-l border-l-line border-t-2 border-t-brand px-3 py-2.5 text-brand-ink";
                    const HEAD_OFF = "flex flex-col border-l border-l-line px-3 py-2.5 text-muted-foreground";
                    const HOUR_HEIGHT = 56;

                    const DAY_NAMES = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
                    const MONTHS = [
                        "January", "February", "March", "April", "May", "June",
                        "July", "August", "September", "October", "November", "December",
                    ];

                    const [offset, setOffset] = pp.state(0);
                    const [view, setView] = pp.state("Work week");
                    const VIEWS = ["Day", "Work week", "Week"];
                    const [meetings, setMeetings] = pp.state(@{Json(&saved_meetings)});
                    const [composeOpen, setComposeOpen] = pp.state(@{start_compose});
                    const [saving, setSaving] = pp.state(false);
                    const [errors, setErrors] = pp.state({});
                    const [formError, setFormError] = pp.state("");
                    const editorForm = pp.ref(null);
                    const newMeetingMarker = pp.ref(null);
                    const eventOpener = pp.ref(null);
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
                    const [pickerMonth, setPickerMonth] = pp.state(@{app_date[..7].to_string()} + '-01');
                    const [pickedDate, setPickedDate] = pp.state(@{app_date});
                    const [pickerFocus, setPickerFocus] = pp.state(@{app_date});
                    const [pickerOpen, setPickerOpen] = pp.state(false);
                    const pickerCloser = pp.ref(null);
                    const pickerElements = pp.ref({});
                    const focusPickerDay = pp.event(() => pickerElements.current[pickerFocus]?.focus());
                    const PICKER_WEEKDAYS = [{name:'Sunday',short:'S'},{name:'Monday',short:'M'},{name:'Tuesday',short:'T'},{name:'Wednesday',short:'W'},{name:'Thursday',short:'T'},{name:'Friday',short:'F'},{name:'Saturday',short:'S'}];
                    pp.effect(() => {
                        if (!pickerOpen) return;
                        const frame = requestAnimationFrame(focusPickerDay);
                        return () => cancelAnimationFrame(frame);
                    }, [pickerOpen, pickerFocus]);
                    const durationMinutes = (new Date(endDate + 'T' + endTime) - new Date(startDate + 'T' + startTime)) / 60000;
                    const durationLabel = Number.isFinite(durationMinutes) && durationMinutes > 0 ? (durationMinutes < 60 ? durationMinutes + 'm' : Math.round(durationMinutes / 6) / 10 + 'h') : '';

                    const scroller = pp.ref(null);
                    const slotElements = pp.ref({});
                    const [selection, setSelection] = pp.state(null);
                    const clockStarted = pp.ref(Date.now());
                    const [clockInstant, setClockInstant] = pp.state(@{app_now.timestamp_millis()});
                    const updateClock = pp.event(() => setClockInstant(@{app_now.timestamp_millis()} + Date.now() - clockStarted.current));
                    pp.effect(() => {
                        const timer = setInterval(updateClock, 30000);
                        return () => clearInterval(timer);
                    }, []);
                    const clockParts = Object.fromEntries(new Intl.DateTimeFormat('en-US', {timeZone:@{app_zone},year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',hourCycle:'h23'}).formatToParts(new Date(clockInstant)).map(part => [part.type, part.value]));
                    const now = new Date(clockParts.year + '-' + clockParts.month + '-' + clockParts.day + 'T' + clockParts.hour + ':' + clockParts.minute);
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
                        const input = editorForm.current?.elements.details;
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
                    // The details dialog is rahti-ui's, which opens from a trigger;
                    // an event on the grid reaches it through the hidden one.
                    const showEvent = pp.event(function showEvent(id) {const meeting = placed.find(item => item.id === id); if (!meeting) return; setSelectedEvent(meeting); eventOpener.current?.click();});
                    function goToday() {setAnchorDate(todayKey); setPickedDate(todayKey); setOffset(0); setSelection(null);}

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
                        const shift = view === "Week" ? -weekday : weekday === 0 ? 1 : 1 - weekday;
                        const first = new Date(gridAnchor.getFullYear(), gridAnchor.getMonth(), gridAnchor.getDate() + shift + offset * 7);
                        const count = view === "Week" ? 7 : 5;
                        for (let index = 0; index < count; index += 1) {
                            list.push(describe(new Date(first.getFullYear(), first.getMonth(), first.getDate() + index)));
                        }
                        return list;
                    }

                    const days = buildDays();
                    const pickerBase = new Date(pickerMonth + 'T12:00');
                    const pickerTitle = MONTHS[pickerBase.getMonth()] + ' ' + pickerBase.getFullYear();
                    const rangeFirst = new Date(days[0].key + 'T12:00');
                    rangeFirst.setDate(rangeFirst.getDate() - rangeFirst.getDay());
                    const rangeLast = new Date(rangeFirst); rangeLast.setDate(rangeLast.getDate() + 6);
                    const activePickerDate = (view === 'Day' ? pickedDate === days[0].key : pickedDate >= dayKey(rangeFirst) && pickedDate <= dayKey(rangeLast)) ? pickedDate : days[0].key;
                    const pickerStart = new Date(pickerBase); pickerStart.setDate(1 - pickerBase.getDay());
                    const pickerCount = Math.ceil((pickerBase.getDay() + new Date(pickerBase.getFullYear(), pickerBase.getMonth() + 1, 0).getDate()) / 7) * 7;
                    const pickerDates = [];
                    for (let index = 0; index < pickerCount; index++) {
                        const date = new Date(pickerStart); date.setDate(date.getDate() + index);
                        const key = dayKey(date);
                        const selected = key === activePickerDate;
                        const inRange = view === 'Day' ? selected : date >= rangeFirst && date <= rangeLast;
                        pickerDates.push({key,number:date.getDate(),selected,today:key === todayKey,label:date.toLocaleDateString('en-US',{weekday:'long',year:'numeric',month:'long',day:'numeric'}),classes:pickerDayClass(date.getMonth() !== pickerBase.getMonth(), inRange, selected)});
                    }

                    const pickerTabDate = pickerDates.some(date => date.key === pickerFocus) ? pickerFocus : pickerMonth;
                    function toggleDatePicker(state) {
                        if (state === 'open') {
                            setPickerMonth(activePickerDate.slice(0,7) + '-01');
                            setPickerFocus(activePickerDate);
                        }
                        setPickerOpen(state === 'open');
                    }
                    function movePickerMonth(amount) {
                        const date = new Date(pickerMonth + 'T12:00');
                        date.setMonth(date.getMonth() + amount);
                        setPickerMonth(dayKey(date));
                    }
                    function chooseCalendarDate(key) {
                        setSelection(null); setPickedDate(key); setAnchorDate(key); setOffset(0);
                        pickerCloser.current?.click();
                    }
                    function pickerKey(event, key) {
                        const moves = {ArrowLeft:-1,ArrowRight:1,ArrowUp:-7,ArrowDown:7};
                        const date = new Date(key + 'T12:00');
                        if (event.key in moves) date.setDate(date.getDate() + moves[event.key]);
                        else if (event.key === 'Home') date.setDate(date.getDate() - date.getDay());
                        else if (event.key === 'End') date.setDate(date.getDate() + 6 - date.getDay());
                        else if (event.key === 'PageUp' || event.key === 'PageDown') {
                            const original = date.getDate(); date.setDate(1);
                            date.setMonth(date.getMonth() + (event.key === 'PageUp' ? -1 : 1));
                            date.setDate(Math.min(original, new Date(date.getFullYear(),date.getMonth()+1,0).getDate()));
                        } else return;
                        event.preventDefault();
                        const next = dayKey(date); setPickerMonth(next.slice(0,7) + '-01'); setPickerFocus(next);
                    }
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
                    for (let half = 0; half < 48; half += 1) {
                        for (let index = 0; index < days.length; index += 1) {
                            const minute = half * 30;
                            cells.push({
                                key: days[index].key + "@" + half,
                                day: days[index].key, half, column:index, minute,
                                time: pad(Math.floor(minute / 60)) + ':' + pad(minute % 60),
                                label: days[index].name + ', ' + days[index].key + ', ' + hourLabel(Math.floor(minute / 60)).replace(' ', ':' + pad(minute % 60) + ' '),
                            });
                        }
                    }

                    const minutesNow = now.getHours() * 60 + now.getMinutes();
                    const nowStyle = "top: " + ((minutesNow / 60) * HOUR_HEIGHT) + "px";
                    const showNow = days.some((day) => day.today);
                    const todayColumn = days.findIndex(day => day.today);
                    const nowGuideStyle = 'width:' + Math.max(0, todayColumn * 100 / days.length) + '%';
                    const nowTodayStyle = 'left:' + Math.max(0, todayColumn * 100 / days.length) + '%;width:' + (100 / days.length) + '%';
                    const selectedSlot = selection || (showNow ? todayKey + '@' + Math.floor(minutesNow / 30) : days[0].key + '@18');
                    pp.effect(() => {
                        if (selection) slotElements.current[selection]?.focus();
                    }, [selection]);

                    function slotKey(event, cell) {
                        if (event.key === 'Enter') { event.preventDefault(); openCompose(cell); return; }
                        const moves = {ArrowUp:[-1,0], ArrowDown:[1,0], ArrowLeft:[0,-1], ArrowRight:[0,1]};
                        const move = moves[event.key];
                        if (!move) return;
                        event.preventDefault();
                        const half = Math.max(0, Math.min(47, cell.half + move[0]));
                        const column = Math.max(0, Math.min(days.length - 1, cell.column + move[1]));
                        const next = cells[half * days.length + column];
                        setSelection(next.key);
                    }

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
                        setSelection(null);
                        setView(value);
                        setOffset(0);
                    }

                    function shiftRange(direction) {
                        setSelection(null);
                        setOffset(value => value + direction);
                    }

                    function openCompose(cell) {
                        const slot = cell || cells.find(item => item.key === selectedSlot);
                        if (slot) {
                            setSelection(slot.key);
                            const end = new Date(slot.day + 'T' + slot.time);
                            end.setMinutes(end.getMinutes() + 30);
                            setStartDate(slot.day); setStartTime(slot.time);
                            setEndDate(dayKey(end)); setEndTime(clockValue(end));
                            setEditorZone(@{app_zone}); setAllDay(false);
                        }
                        setErrors({});
                        setFormError("");
                        setComposeOpen(true);
                    }
                    const editorWasOpen = pp.ref(false);
                    const newMeetingButton = () => newMeetingMarker.current?.closest("button");
                    const focusEditor = pp.event(() => (composeOpen ? editorForm.current?.elements.title : newMeetingButton())?.focus());
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
                        newMeetingButton()?.focus();
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
                            newMeetingButton()?.focus();
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
