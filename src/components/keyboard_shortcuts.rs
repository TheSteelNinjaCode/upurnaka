//! Teams-style shortcut catalog with availability for implemented app actions.
use crate::components::aria::{bound, labelled, named};
use crate::components::icon::Icon;
use crate::components::rahti_ui::attrs::{Attrs, ui_expr};
use crate::components::rahti_ui::button::{Button, ButtonSize, ButtonVariant, button_variants};
use crate::components::rahti_ui::dialog::{DialogClose, DialogTitle};
use crate::components::rahti_ui::input_group::{InputGroup, InputGroupAddon, InputGroupInput};
use crate::components::rahti_ui::kbd::Kbd;
use crate::components::rahti_ui::label::Label;
use crate::components::rahti_ui::select::{Select, SelectItem};
use crate::rahti::{Html, component, html};

/// A section heading that folds its shortcuts away.
const SECTION_TOGGLE: &str = "shortcuts-section-toggle mb-2 h-auto! w-fit gap-3.5 px-2! py-2.5 \
                              text-base font-semibold";

#[component]
pub fn KeyboardShortcuts() -> Html {
    let close = button_variants(
        ButtonVariant::Ghost,
        ButtonSize::Icon,
        "text-muted-foreground",
    );
    html! {
        <div class="flex h-full min-h-0 flex-col text-base leading-[1.45]">
            <header class="shrink-0 px-[22px] pb-3 pt-6 min-[901px]:px-[34px] min-[901px]:pt-8">
                <div class="shortcuts-title mb-[22px] flex items-center justify-between gap-5">
                    <DialogTitle class="text-xl font-bold min-[651px]:text-[22px]">
                        "Keyboard shortcuts"
                    </DialogTitle>
                    <DialogClose
                        class=@{close.as_str()}
                        attrs=@{labelled("Close keyboard shortcuts")}
                    >
                        <Icon name="close" class="size-6" />
                    </DialogClose>
                </div>
                <div
                    class="flex flex-col items-stretch gap-3.5 min-[651px]:flex-row min-[651px]:items-end min-[651px]:justify-between min-[651px]:gap-7"
                >
                    <div class="flex w-full flex-col gap-2 min-[651px]:w-3/5 min-[901px]:w-1/2">
                        <Label for="shortcut-profile">"Use shortcuts from other apps"</Label>
                        <div class="flex gap-3.5">
                            <Select
                                id="shortcut-profile"
                                class="h-11 w-full min-w-0"
                                attrs=@{named("Shortcut profile")}
                            >
                                <SelectItem value="teams" selected>
                                    "Teams only (default)"
                                </SelectItem>
                            </Select>
                            <Button
                                variant="secondary"
                                size="lg"
                                class="h-11! min-w-[76px] min-[901px]:min-w-[124px]"
                                disabled
                                attrs=@{Attrs::new().set("title", "The default shortcut profile is already applied")}
                            >
                                "Apply"
                            </Button>
                        </div>
                    </div>
                    <InputGroup class="h-11! w-full min-[651px]:w-2/5 min-[901px]:w-[34%]">
                        <InputGroupAddon>
                            <Icon name="search" class="size-5" />
                        </InputGroupAddon>
                        <InputGroupInput
                            type="search"
                            placeholder="Filter shortcuts"
                            attrs=@{named("Filter shortcuts")}
                            value={filter}
                            oninput={setFilter(target.value)}
                        />
                    </InputGroup>
                </div>
            </header>
            <div
                class="scroll-thin min-h-0 flex-1 overflow-y-auto overscroll-contain px-[22px] min-[901px]:px-[34px] focus-visible:outline-2 focus-visible:outline-offset-3 focus-visible:outline-muted-foreground"
                tabindex="0"
                aria-label="Shortcut categories"
            >
                <template pp-for="section in shown">
                    <section
                        key={section.title}
                        class="shortcuts-section border-b border-edge pb-[18px] pt-3 last-of-type:border-b-0"
                    >
                        <Button
                            variant="ghost"
                            class=@{SECTION_TOGGLE}
                            attrs=@{bound("aria-expanded", ui_expr!(aria-expanded={!collapsed.includes(section.title) || searching}))}
                            value={section.title}
                            onclick={toggleSection(event.currentTarget.value)}
                        >
                            <span
                                class={collapsed.includes(section.title) && !searching ? "inline-flex -rotate-90" : "inline-flex"}
                            >
                                <Icon name="chevron-down" class="size-5" />
                            </span>
                            {section.title}
                        </Button>
                        <div hidden={collapsed.includes(section.title) && !searching}>
                            <p
                                class="mb-1.5 ml-2 mr-7 text-sm text-muted-foreground min-[651px]:mx-7"
                                hidden={section.title !== "Message actions"}
                            >
                                "Single-character shortcuts are supported only for actions in this section."
                            </p>
                            <div
                                class="grid grid-cols-1 gap-0 px-2 min-[651px]:grid-cols-2 min-[651px]:gap-6 min-[901px]:gap-9 min-[901px]:px-7"
                            >
                                <template pp-for="column in section.columns">
                                    <dl key={column.id}>
                                        <template pp-for="shortcut in column.items">
                                            <div
                                                class="shortcuts-row grid min-h-[60px] grid-cols-[minmax(0,1fr)_minmax(115px,1fr)] items-center gap-2.5 text-sm min-[901px]:grid-cols-[minmax(0,1.3fr)_minmax(150px,1fr)] min-[901px]:gap-4 min-[901px]:text-base"
                                                key={shortcut[0]}
                                            >
                                                <dt class="py-2">
                                                    {shortcut[0]}
                                                    <span
                                                        class="mt-0.5 block text-[11px] leading-[1.3] text-subtle"
                                                        hidden={availableActions.has(shortcut[0])}
                                                    >
                                                        "Not available yet"
                                                    </span>
                                                </dt>
                                                <dd>
                                                    <Kbd
                                                        class="h-auto! w-full! rounded-full! bg-surface! px-2 py-2.5 text-sm! text-ink!"
                                                    >
                                                        {shortcut[1]}
                                                    </Kbd>
                                                </dd>
                                            </div>
                                        </template>
                                    </dl>
                                </template>
                            </div>
                        </div>
                    </section>
                </template>
                <p
                    class="shortcuts-empty px-4 py-[60px] text-center text-muted-foreground"
                    role="status"
                    hidden={shown.length > 0}
                >
                    "No shortcuts found. Try another action or key combination."
                </p>
            </div>
            <footer
                class="flex shrink-0 flex-wrap items-center gap-2 px-[30px] py-[18px] text-sm min-[651px]:gap-3 min-[901px]:px-11 min-[901px]:py-[22px] min-[901px]:text-base"
            >
                <a
                    href="https://support.microsoft.com/office/keyboard-shortcuts-for-microsoft-teams-2e8e2a70-e8d8-4a19-949b-4c36dd5292d2"
                    class="text-brand-ink underline underline-offset-2 focus-visible:outline-2 focus-visible:outline-offset-3 focus-visible:outline-muted-foreground"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "See shortcuts for all platforms"
                </a>
                <span class="hidden h-6 w-px bg-edge min-[651px]:block" aria-hidden="true"></span>
                <a
                    href="https://support.microsoft.com/office/office-accessibility-center-resources-for-people-with-disabilities-ecab0fcf-d143-4fe8-a2ff-6cd596bddc6d"
                    class="text-brand-ink underline underline-offset-2 focus-visible:outline-2 focus-visible:outline-offset-3 focus-visible:outline-muted-foreground"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "Office Accessibility Center"
                </a>
            </footer>
            <script>r##"
                const [filter, setFilter] = pp.state("");
                const [collapsed, setCollapsed] = pp.state([]);
                const searching = filter.trim().length > 0;
                function normalize(value) { return value.toLowerCase().replace(/\s+/g, ""); }
                const toggleSection = pp.event(function toggleSection(title) {
                    setCollapsed(previous => previous.includes(title) ? previous.filter(item => item !== title) : [...previous, title]);
                });
                const availableActions = new Set([
                    "Show keyboard shortcuts", "Start new chat", "Open Search", "Show commands", "Go to a specific chat or channel",
                    "Zoom in", "Zoom out", "Reset zoom level", "Toggle app bar",
                    "Go to compose box", "Go back", "Go forward", "Move focus to app list", "Move focus to message pane",
                    "Open 1st app on app bar", "Open 2nd app on app bar", "Open 3rd app on app bar", "Open 4th app on app bar",
                    "Open 5th app on app bar", "Open filter",
                    "Send (expanded compose box)", "Start new line", "Expand compose box", "Undo", "Redo", "Cut", "Copy", "Paste", "Select all",
                    "Find in current chat or channel", "Jump to last read or newest message", "Clear all filters",
                    "See all chat conversations", "See all unread items", "See all unmuted items", "See all meeting items", "Collapse all conversation folders",
                ]);
                const catalog = [
                    {
                        title: "General",
                        columns: [
                            [
                                ["Show keyboard shortcuts", "Ctrl + ."],
                                ["Open Search", "Ctrl + E"],
                                ["Show commands", "Ctrl + /"],
                                ["Go to a specific chat or channel", "Ctrl + G"],
                                ["Start new chat", "Ctrl + N"],
                                ["Start chat in new window", "Ctrl + Shift + N"],
                                ["Open existing conversation in new window", "Ctrl + O"],
                                ["Open filter", "Ctrl + Shift + F"],
                            ],
                            [
                                ["Open Settings", "Ctrl + ,"],
                                ["Open Help", "F1"],
                                ["Report a problem", "Ctrl + Alt + Shift + R"],
                                ["Zoom in", "Ctrl + +"],
                                ["Zoom out", "Ctrl + -"],
                                ["Reset zoom level", "Ctrl + 0"],
                                ["Toggle pin window on top", "Ctrl + Shift + T"],
                                ["Toggle app bar", "Ctrl + \\"],
                            ],
                        ],
                    },
                    {
                        title: "Navigation",
                        columns: [
                            [
                                ["Go to compose box", "Ctrl + R"],
                                ["Go back", "Alt + Left"],
                                ["Go forward", "Alt + Right"],
                                ["Open history menu", "Ctrl + H"],
                                ["Move focus to app list", "Ctrl + L"],
                                ["Move focus to message pane", "Ctrl + M"],
                                ["Move focus to toast", "Ctrl + Alt + T"],
                                ["Move focus to notification", "Win + Shift + Y"],
                                ["Open 1st app on app bar", "Ctrl + 1"],
                                ["Open 2nd app on app bar", "Ctrl + 2"],
                            ],
                            [
                                ["Open 3rd app on app bar", "Ctrl + 3"],
                                ["Open 4th app on app bar", "Ctrl + 4"],
                                ["Open 5th app on app bar", "Ctrl + 5"],
                                ["Focus resizable divider", "Ctrl + Alt + Enter"],
                                ["Reset slots to default size", "Ctrl + Shift + Enter"],
                            ],
                        ],
                    },
                    {
                        title: "Compose",
                        columns: [
                            [
                                ["Send (expanded compose box)", "Ctrl + Enter"],
                                ["Start new line", "Shift + Enter"],
                                ["Expand compose box", "Ctrl + Shift + X"],
                                ["Attach file", "Alt + Shift + O"],
                                ["Mark message as important", "Ctrl + Shift + I"],
                                ["Undo", "Ctrl + Z"],
                                ["Redo", "Ctrl + Y"],
                                ["Cut", "Ctrl + X"],
                                ["Copy", "Ctrl + C"],
                                ["Paste", "Ctrl + V"],
                                ["Select all", "Ctrl + A"],
                                ["Apply bold style", "Ctrl + B"],
                                ["Apply italic style", "Ctrl + I"],
                                ["Apply underline style", "Ctrl + U"],
                                ["Apply strikethrough style", "Ctrl + Alt + X"],
                            ],
                            [
                                ["Insert link", "Ctrl + K"],
                                ["Insert inline code", "Ctrl + Shift + C"],
                                ["Insert code block", "Ctrl + Shift + B"],
                                ["Apply heading 1 style", "Ctrl + Alt + 1"],
                                ["Apply heading 2 style", "Ctrl + Alt + 2"],
                                ["Apply heading 3 style", "Ctrl + Alt + 3"],
                                ["Apply paragraph style", "Ctrl + Alt + P"],
                                ["Apply quote style", "Ctrl + Alt + 4"],
                                ["Insert numbered list", "Ctrl + Shift + 7"],
                                ["Insert bulleted list", "Ctrl + Shift + 8"],
                                ["Add Loop paragraph", "Ctrl + Alt + L"],
                                ["Rewrite with Copilot", "Alt + A"],
                                ["Insert table row or column", "Ctrl + Alt + ="],
                                ["Remove table row or column", "Ctrl + Alt + -"],
                            ],
                        ],
                    },
                    {
                        title: "Messaging",
                        columns: [
                            [
                                ["Find in current chat or channel", "Ctrl + F"],
                                ["Reply to the last message", "Alt + Shift + R"],
                                ["React to the last message", "Ctrl + Alt + R"],
                                ["Jump to last read or newest message", "Ctrl + J"],
                                ["Clear all filters", "Ctrl + Alt + Z"],
                                ["See all chat conversations", "Ctrl + Alt + C"],
                            ],
                            [
                                ["See all channel conversations", "Ctrl + Alt + A"],
                                ["See all unread items", "Ctrl + Alt + U"],
                                ["See all unmuted items", "Ctrl + Alt + Q"],
                                ["See all meeting items", "Ctrl + Alt + B"],
                                ["Collapse all conversation folders", "Alt + Q"],
                                ["Toggle Copilot pane", "Alt + C"],
                            ],
                        ],
                    },
                    {
                        title: "Message actions",
                        columns: [
                            [
                                ["Reply with quote", "Q"],
                                ["Forward message", "F"],
                                ["Focus emoji reactions", "R"],
                                ["Open thread", "T"],
                            ],
                            [
                                ["Mark message unread", "U"],
                                ["Save message", "S"],
                                ["Pin message", "P"],
                                ["Delete message", "Delete"],
                            ],
                        ],
                    },
                    {
                        title: "Meetings, Calls and Calendar",
                        columns: [
                            [
                                ["Toggle mute", "Ctrl + Shift + M"],
                                ["Toggle video", "Ctrl + Shift + O"],
                                ["Toggle share content tray", "Ctrl + Shift + E"],
                                ["Raise or lower your hand", "Ctrl + Shift + K"],
                                ["Toggle live captions", "Alt + Shift + C"],
                                ["Leave meeting or call", "Ctrl + Shift + H"],
                                ["Temporarily unmute", "Ctrl + Space"],
                                ["Toggle speaker", "Ctrl + Shift + U"],
                                ["Go to sharing toolbar", "Ctrl + Shift + Space"],
                                ["Toggle background blur", "Ctrl + Shift + P"],
                                ["Toggle full screen", "F11"],
                                ["Announce raised hands (screen reader)", "Ctrl + Shift + L"],
                                ["Announce active speaker (screen reader)", "Ctrl + Alt + Shift + S"],
                                ["Admit people from lobby notification", "Ctrl + Shift + Y"],
                                ["Start audio call", "Alt + Shift + A"],
                                ["Start video call", "Alt + Shift + V"],
                                ["Accept audio call", "Ctrl + Shift + S"],
                            ],
                            [
                                ["Accept video call", "Ctrl + Shift + A"],
                                ["Accept screen share", "Ctrl + Shift + A"],
                                ["Decline call", "Ctrl + Shift + D"],
                                ["Decline screen share", "Ctrl + Shift + D"],
                                ["Join from meeting started toast", "Ctrl + Shift + J"],
                                ["Open meeting chat from Meeting start notification", "Ctrl + Shift + R"],
                                ["Zoom in shared content", "Alt + Shift + +"],
                                ["Zoom out shared content", "Alt + Shift + -"],
                                ["Reset zoom for shared content", "Alt + Shift + 0"],
                                ["Pan shared content up", "Alt + Shift + Up"],
                                ["Pan shared content down", "Alt + Shift + Down"],
                                ["Pan shared content left", "Alt + Shift + Left"],
                                ["Pan shared content right", "Alt + Shift + Right"],
                                ["Schedule a meeting", "Alt + N"],
                                ["Save/send meeting request", "Ctrl + S"],
                                ["Join focused meeting", "Ctrl + J"],
                            ],
                        ],
                    },
                    {
                        title: "Debug",
                        columns: [
                            [
                                ["Download log files", "Ctrl + Alt + Shift + 1"],
                            ],
                            [
                            ],
                        ],
                    },
                ];
                const needle = normalize(filter);
                const shown = catalog.map(section => ({
                    title: section.title,
                    columns: section.columns.map((items, id) => ({id, items: items.filter(shortcut =>
                        normalize(section.title).includes(needle) || normalize(shortcut.join(" ")).includes(needle)
                    )}))
                })).filter(section => section.columns.some(column => column.items.length));
            "##</script>
        </div>
    }
}
