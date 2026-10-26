//! Teams-style shortcut catalog with availability for implemented app actions.
use crate::components::icon::Icon;
use crate::rahti::{Html, component, html};

#[component]
pub fn KeyboardShortcuts() -> Html {
    html! {
        <div class="shortcuts-content">
            <header class="shortcuts-header">
                <div class="shortcuts-title">
                    <h2>"Keyboard shortcuts"</h2>
                    <form method="dialog">
                        <button type="submit" aria-label="Close keyboard shortcuts">
                            <Icon name="close" class="h-6 w-6" />
                        </button>
                    </form>
                </div>
                <div class="shortcuts-controls">
                    <div class="shortcuts-profile">
                        <label for={profileId}>"Use shortcuts from other apps"</label>
                        <div class="shortcuts-profile-row">
                            <select id={profileId} aria-label="Shortcut profile">
                                <option value="teams">"Teams only (default)"</option>
                            </select>
                            <button
                                type="button"
                                disabled
                                title="The default shortcut profile is already applied"
                            >
                                "Apply"
                            </button>
                        </div>
                    </div>
                    <div class="shortcuts-search">
                        <Icon name="search" class="h-5 w-5" />
                        <input
                            type="search"
                            aria-label="Filter shortcuts"
                            placeholder="Filter shortcuts"
                            value={filter}
                            oninput={setFilter(target.value)}
                        />
                    </div>
                </div>
            </header>
            <div class="shortcuts-scroll scroll-thin" tabindex="0" aria-label="Shortcut categories">
                <template pp-for="section in shown">
                    <section class="shortcuts-section" key={section.title}>
                        <button
                            type="button"
                            class="shortcuts-section-toggle"
                            aria-expanded={!collapsed.includes(section.title) || searching}
                            onclick={toggleSection(section.title)}
                        >
                            <span
                                class={collapsed.includes(section.title) && !searching ? "shortcuts-chevron collapsed" : "shortcuts-chevron"}
                            >
                                <Icon name="chevron-down" class="h-5 w-5" />
                            </span>
                            {section.title}
                        </button>
                        <div hidden={collapsed.includes(section.title) && !searching}>
                            <p class="shortcuts-note" hidden={section.title !== "Message actions"}>
                                "Single-character shortcuts are supported only for actions in this section."
                            </p>
                            <div class="shortcuts-columns">
                                <template pp-for="column in section.columns">
                                    <dl class="shortcuts-column" key={column.id}>
                                        <template pp-for="shortcut in column.items">
                                            <div class="shortcuts-row" key={shortcut[0]}>
                                                <dt>
                                                    {shortcut[0]}
                                                    <span
                                                        class="shortcuts-unavailable"
                                                        hidden={availableActions.has(shortcut[0])}
                                                    >
                                                        "Not available yet"
                                                    </span>
                                                </dt>
                                                <dd>
                                                    <kbd>{shortcut[1]}</kbd>
                                                </dd>
                                            </div>
                                        </template>
                                    </dl>
                                </template>
                            </div>
                        </div>
                    </section>
                </template>
                <p class="shortcuts-empty" role="status" hidden={shown.length > 0}>
                    "No shortcuts found. Try another action or key combination."
                </p>
            </div>
            <footer class="shortcuts-footer">
                <a
                    href="https://support.microsoft.com/office/keyboard-shortcuts-for-microsoft-teams-2e8e2a70-e8d8-4a19-949b-4c36dd5292d2"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "See shortcuts for all platforms"
                </a>
                <span aria-hidden="true"></span>
                <a
                    href="https://support.microsoft.com/office/office-accessibility-center-resources-for-people-with-disabilities-ecab0fcf-d143-4fe8-a2ff-6cd596bddc6d"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    "Office Accessibility Center"
                </a>
            </footer>
            <script>r##"
                const [filter, setFilter] = pp.state("");
                const [collapsed, setCollapsed] = pp.state([]);
                const profileId = pp.id();
                const searching = filter.trim().length > 0;
                function normalize(value) { return value.toLowerCase().replace(/\s+/g, ""); }
                function toggleSection(title) {
                    setCollapsed(previous => previous.includes(title) ? previous.filter(item => item !== title) : [...previous, title]);
                }
                const availableActions = new Set([
                    "Show keyboard shortcuts", "Open Search", "Show commands", "Go to a specific chat or channel",
                    "Zoom in", "Zoom out", "Reset zoom level", "Toggle app bar",
                    "Go to compose box", "Go back", "Go forward", "Move focus to app list", "Move focus to message pane",
                    "Open 1st app on app bar", "Open 2nd app on app bar", "Open 3rd app on app bar", "Open 4th app on app bar",
                    "Open 5th app on app bar", "Open 6th app on app bar", "Open 7th app on app bar", "Open filter",
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
                                ["Open 6th app on app bar", "Ctrl + 6"],
                                ["Open 7th app on app bar", "Ctrl + 7"],
                                ["Open 8th app on app bar", "Ctrl + 8"],
                                ["Open 9th app on app bar", "Ctrl + 9"],
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
            <style>r##"
                .shortcuts-dialog { width:calc(100vw - 32px);max-width:1080px;height:calc(100dvh - 32px);max-height:1400px;margin:auto;padding:0;border:0;border-radius:12px;background:#292929;color:#f5f5f5;box-shadow:0 24px 80px #0008; }
                .shortcuts-dialog::backdrop { background:#0008; }
                .shortcuts-content { display:flex;flex-direction:column;height:100%;min-height:0;font-size:16px;line-height:1.45; }
                .shortcuts-header { flex-shrink:0;padding:32px 34px 12px; }
                .shortcuts-title { display:flex;align-items:center;justify-content:space-between;margin-bottom:22px;gap:20px; }
                .shortcuts-title h2 { font-size:22px;font-weight:700; }
                .shortcuts-title button { display:grid;place-items:center;width:32px;height:32px;color:#ccc;border-radius:4px;cursor:pointer; }
                .shortcuts-title button:hover { background:#3d3d3d; }
                .shortcuts-controls { display:flex;align-items:flex-end;justify-content:space-between;gap:28px; }
                .shortcuts-profile { width:50%; }
                .shortcuts-profile label { display:block;margin-bottom:5px; }
                .shortcuts-profile-row { display:flex;gap:14px; }
                .shortcuts-profile select { width:100%;min-width:0;height:44px;padding:0 14px;color:inherit;background:#292929;border:1px solid #666;border-radius:5px;font:inherit; }
                .shortcuts-profile-row>button { height:44px;min-width:124px;padding:0 20px;border-radius:5px;background:#141414;color:#666;font-weight:600;cursor:not-allowed; }
                .shortcuts-search { display:flex;align-items:center;gap:10px;width:34%;height:44px;padding:0 16px;border-radius:6px;background:#1f1f1f;color:#aaa; }
                .shortcuts-search:focus-within { outline:2px solid #a6a7ef;outline-offset:1px; }
                .shortcuts-search input { width:100%;min-width:0;background:transparent;border:0;outline:none;color:#f5f5f5;font:inherit; }
                .shortcuts-scroll { flex:1;min-height:0;overflow-y:auto;overscroll-behavior:contain;padding:0 34px; }
                .shortcuts-section { border-bottom:1px solid #515151;padding:12px 0 18px; }
                .shortcuts-section:last-of-type { border-bottom:0; }
                .shortcuts-section-toggle { display:flex;align-items:center;gap:14px;padding:10px 8px;margin-bottom:8px;font-weight:600;border-radius:4px;cursor:pointer; }
                .shortcuts-chevron { display:inline-flex; }
                .shortcuts-chevron.collapsed { transform:rotate(-90deg); }
                .shortcuts-columns { display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:36px;padding:0 28px; }
                .shortcuts-row { display:grid;grid-template-columns:minmax(0,1.3fr) minmax(150px,1fr);align-items:center;gap:16px;min-height:60px; }
                .shortcuts-row dt { padding:8px 0; }
                .shortcuts-row dd { margin:0; }
                .shortcuts-unavailable { display:block;color:#aaa;font-size:11px;line-height:1.3;margin-top:2px; }
                .shortcuts-row kbd { display:block;padding:10px 8px;border-radius:999px;background:#1f1f1f;text-align:center;white-space:nowrap;font-family:inherit;font-size:inherit; }
                .shortcuts-note { margin:0 28px 6px;color:#ccc;font-size:14px; }
                .shortcuts-empty { padding:60px 16px;text-align:center;color:#ccc; }
                .shortcuts-footer { display:flex;align-items:center;flex-wrap:wrap;gap:12px;flex-shrink:0;padding:22px 44px; }
                .shortcuts-footer a { color:#9992f5;text-decoration:underline;text-underline-offset:2px; }
                .shortcuts-footer>span { height:24px;width:1px;background:#1f1f1f; }
                .shortcuts-dialog :focus-visible { outline:2px solid #ddd;outline-offset:3px; }
                .shortcuts-dialog [hidden] { display:none; }
                @media(max-width:900px) { .shortcuts-header { padding:24px 22px 12px; }.shortcuts-scroll { padding:0 22px; }.shortcuts-profile { width:60%; }.shortcuts-search { width:40%; }.shortcuts-columns { gap:24px;padding:0 8px; }.shortcuts-row { grid-template-columns:minmax(0,1fr) minmax(115px,1fr);gap:10px;font-size:14px; }.shortcuts-profile-row>button { min-width:76px; }.shortcuts-footer { padding:18px 30px;font-size:14px; } }
                @media(max-width:650px) { .shortcuts-controls { flex-direction:column;align-items:stretch;gap:14px; }.shortcuts-profile,.shortcuts-search { width:100%; }.shortcuts-columns { grid-template-columns:minmax(0,1fr);gap:0; }.shortcuts-title h2 { font-size:20px; }.shortcuts-footer>span { display:none; }.shortcuts-footer { gap:8px; }.shortcuts-note { margin-left:8px; } }
            "##</style>
        </div>
    }
}
