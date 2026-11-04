use crate::{
    components::{icon::Icon, notification_sound::NotificationSound},
    rahti::{Html, component, html},
};

#[component]
pub fn NotificationSettings() -> Html {
    html! {
        <div class="notification-settings">
            <NotificationSound />
            <h3 class="notification-heading">"General"</h3>
            <section class="settings-card notification-setting">
                <Icon name="bell-off" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Mute all notifications (except for calls and meetings)"</h3>
                </div>
                <label class="accessibility-control">
                    <span aria-hidden="true">{prefs.notifyMuteAll ? "On" : "Off"}</span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Mute all notifications"
                        checked={prefs.notifyMuteAll}
                        onchange={settings.update("notifyMuteAll", target.checked)}
                    />
                </label>
            </section>
            <section class="settings-card notification-setting">
                <Icon name="bell" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>
                        "When my device's Do not disturb is on, don't show Upurnaka notifications"
                    </h3>
                    <p>
                        "Browsers don't share your device's Do not disturb status, so Upurnaka can't follow it. Use Mute all notifications instead."
                    </p>
                </div>
                <label class="accessibility-control">
                    <span aria-hidden="true">"Off"</span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Follow device Do not disturb"
                        disabled
                    />
                </label>
            </section>
            <section class="settings-card">
                <div class="settings-card-heading">
                    <Icon name="speaker" class="size-5" />
                    <h3>"Sound"</h3>
                </div>
                <div class="notification-check-row">
                    <label class="notification-check">
                        <input
                            type="checkbox"
                            checked={prefs.notifySound}
                            onchange={settings.update("notifySound", target.checked)}
                        />
                        <span>"Play sounds with notifications"</span>
                    </label>
                    <span class="notification-sound-picker">
                        <select
                            class="settings-select"
                            aria-label="Notification sound"
                            value={prefs.notifySoundName}
                            disabled={!prefs.notifySound}
                            onchange={settings.update("notifySoundName", target.value)}
                        >
                            <option value="default">"Default"</option>
                            <option value="chime">"Chime"</option>
                            <option value="ping">"Ping"</option>
                            <option value="pop">"Pop"</option>
                        </select>
                        <button
                            type="button"
                            class="notification-play"
                            aria-label="Play notification sound"
                            disabled={!prefs.notifySound}
                            onclick={previewSound()}
                        >
                            <Icon name="play" class="size-4" />
                        </button>
                    </span>
                </div>
                <label class="notification-check">
                    <input
                        type="checkbox"
                        checked={prefs.notifyCallSound}
                        onchange={settings.update("notifyCallSound", target.checked)}
                    />
                    <span>
                        "Play sounds with incoming calls, requests to join, and meeting start notifications"
                        <small>
                            "Meeting organizers hear a sound when someone waits in the lobby."
                        </small>
                    </span>
                </label>
                <label class="notification-check is-disabled">
                    <input type="checkbox" disabled />
                    <span>
                        "Play sounds when I'm busy or in a meeting"
                        <small>
                            "Not available yet: other notifications are not shown during meetings."
                        </small>
                    </span>
                </label>
            </section>
            <section class="settings-card">
                <div class="settings-card-heading notification-display-heading">
                    <Icon name="eye" class="size-5" />
                    <h3>"Display"</h3>
                    <button type="button" class="settings-secondary" onclick={testNotification()}>
                        "Test notification"
                    </button>
                </div>
                <p
                    class="settings-detail notification-muted-note"
                    role="status"
                    hidden={!testMuted}
                >
                    "Notifications are muted. Turn off Mute all notifications to see a test notification."
                </p>
                <label class="notification-check">
                    <input
                        type="checkbox"
                        checked={prefs.notifyPreviews}
                        onchange={settings.update("notifyPreviews", target.checked)}
                    />
                    <span>"Show message and content previews in notifications"</span>
                </label>
                <label class="notification-check is-disabled">
                    <input type="checkbox" disabled />
                    <span>
                        "Show notifications during calls and meetings"
                        <small>"Not available yet: meetings show their own in-call alerts."</small>
                    </span>
                </label>
                <label class="notification-check">
                    <input
                        type="checkbox"
                        checked={prefs.notifyFlashTab}
                        onchange={settings.update("notifyFlashTab", target.checked)}
                    />
                    <span>
                        "Flash the browser tab when notifications are received"
                        <small>"The tab title flashes while Upurnaka is in the background."</small>
                    </span>
                </label>
                <label class="settings-row notification-row">
                    <span>"Dismiss notifications after this amount of time"</span>
                    <select
                        class="settings-select"
                        aria-label="Dismiss notifications after"
                        value={prefs.notifyDismissAfter}
                        onchange={settings.update("notifyDismissAfter", target.value)}
                    >
                        <option value="5">"5 seconds (Default)"</option>
                        <option value="10">"10 seconds"</option>
                        <option value="20">"20 seconds"</option>
                        <option value="30">"30 seconds"</option>
                    </select>
                </label>
                <p class="notification-label">"Position on the screen"</p>
                <div class="settings-choices" role="radiogroup" aria-label="Position on the screen">
                    <template pp-for="position in positions">
                        <label
                            key={position.key}
                            class={"settings-choice notification-choice " + (prefs.notifyPosition === position.key ? "is-selected" : "")}
                        >
                            <span
                                class={"notification-position-preview at-" + position.key}
                                aria-hidden="true"
                            >
                                <i></i>
                            </span>
                            <span class="flex items-center justify-between gap-3">
                                {position.label}
                                <input
                                    type="radio"
                                    name="notification-position"
                                    value={position.key}
                                    checked={prefs.notifyPosition === position.key}
                                    onchange={settings.update("notifyPosition", target.value)}
                                />
                            </span>
                        </label>
                    </template>
                </div>
                <p class="notification-label">"Size"</p>
                <div class="settings-choices" role="radiogroup" aria-label="Notification size">
                    <template pp-for="size in sizes">
                        <label
                            key={size.key}
                            class={"settings-choice notification-choice " + (prefs.notifySize === size.key ? "is-selected" : "")}
                        >
                            <span
                                class={"notification-size-preview size-" + size.key}
                                aria-hidden="true"
                            >
                                <i>
                                    <b></b>
                                    <s></s>
                                    <u></u>
                                </i>
                            </span>
                            <span class="flex items-center justify-between gap-3">
                                {size.label}
                                <input
                                    type="radio"
                                    name="notification-size"
                                    value={size.key}
                                    checked={prefs.notifySize === size.key}
                                    onchange={settings.update("notifySize", target.value)}
                                />
                            </span>
                        </label>
                    </template>
                </div>
            </section>
            <section class="settings-card notification-setting">
                <Icon name="mail" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Missed activity emails"</h3>
                    <p>"Email delivery is not configured for this workspace."</p>
                </div>
                <select class="settings-select" aria-label="Missed activity emails" disabled>
                    <option>"Not available yet"</option>
                </select>
            </section>
            <h3 class="notification-heading">"Chat"</h3>
            <p class="notification-section-note">
                "Chat messages are not delivered to other people yet, so there are no chat notifications to configure."
            </p>
            <section class="settings-card notification-setting">
                <Icon name="chat" class="size-5" />
                <h3 class="min-w-0 flex-1">"Chat message notifications"</h3>
                <select class="settings-select" aria-label="Chat message notifications" disabled>
                    <option>"Not available yet"</option>
                </select>
            </section>
            <section class="settings-card notification-setting">
                <Icon name="at" class="size-5" />
                <h3 class="min-w-0 flex-1">"@Mentions"</h3>
                <select class="settings-select" aria-label="Mention notifications" disabled>
                    <option>"Not available yet"</option>
                </select>
            </section>
            <section class="settings-card notification-setting">
                <Icon name="thumbs-up" class="size-5" />
                <h3 class="min-w-0 flex-1">"Likes and reactions"</h3>
                <select class="settings-select" aria-label="Reaction notifications" disabled>
                    <option>"Not available yet"</option>
                </select>
            </section>
            <h3 class="notification-heading">"Meetings"</h3>
            <section class="settings-card">
                <div class="settings-card-heading">
                    <Icon name="calendar" class="size-5" />
                    <div>
                        <h3>"Meetings"</h3>
                        <p>
                            "Meeting reminders are not available yet. Join requests still appear inside the meeting for organizers."
                        </p>
                    </div>
                </div>
                <label class="settings-row notification-row">
                    <span>"Meeting start notifications"</span>
                    <span class="accessibility-control">
                        <span aria-hidden="true">"Off"</span>
                        <input
                            class="settings-switch"
                            type="checkbox"
                            role="switch"
                            aria-label="Meeting start notifications"
                            disabled
                        />
                    </span>
                </label>
                <label class="settings-row notification-row">
                    <span>"Show meeting join bar"</span>
                    <span class="accessibility-control">
                        <span aria-hidden="true">"Off"</span>
                        <input
                            class="settings-switch"
                            type="checkbox"
                            role="switch"
                            aria-label="Show meeting join bar"
                            disabled
                        />
                    </span>
                </label>
                <label class="settings-row notification-row">
                    <span>"Meeting chat notifications"</span>
                    <select
                        class="settings-select"
                        aria-label="Meeting chat notifications"
                        disabled
                    >
                        <option>"Not available yet"</option>
                    </select>
                </label>
                <label class="settings-row notification-row">
                    <span>"Meet now start notifications"</span>
                    <span class="accessibility-control">
                        <span aria-hidden="true">"Off"</span>
                        <input
                            class="settings-switch"
                            type="checkbox"
                            role="switch"
                            aria-label="Meet now start notifications"
                            disabled
                        />
                    </span>
                </label>
            </section>
            <div
                class={"notification-toast at-" + prefs.notifyPosition + " size-" + prefs.notifySize}
                role="status"
                aria-live="polite"
                hidden={!toast}
                pp-ref={toastElement}
                popover="manual"
            >
                <span class="notification-toast-avatar" aria-hidden="true">"U"</span>
                <span class="min-w-0 flex-1">
                    <strong>"Upurnaka"</strong>
                    <span class="notification-toast-body">
                        {prefs.notifyPreviews ? "This is a test notification. Your notifications will look like this." : "You have a new notification."}
                    </span>
                </span>
                <button
                    type="button"
                    class="notification-toast-close"
                    aria-label="Dismiss notification"
                    onclick={dismiss()}
                >
                    <Icon name="close" class="size-4" />
                </button>
            </div>
            <script>r##"
                const settings = pp.context(window.UpurnakaSettingsContext);
                const prefs = settings.preferences;
                const positions = [
                    {key:'bottom-right', label:'Bottom right'},
                    {key:'top-right', label:'Top right'},
                    {key:'bottom-left', label:'Bottom left'},
                    {key:'top-left', label:'Top left'}
                ];
                const sizes = [{key:'comfy', label:'Comfy'}, {key:'compact', label:'Compact'}];
                const [toast, setToast] = pp.state(0);
                const toastElement = pp.ref(null);
                // The browser's top layer escapes the page's stacking context
                // without moving focus away from the settings control.
                pp.effect(() => {
                    const element = toastElement.current;
                    if (toast) element?.showPopover();
                    else element?.hidePopover();
                }, [toast]);
                const [testMuted, setTestMuted] = pp.state(false);
                function previewSound() { window.UpurnakaNotificationSound?.play(prefs.notifySoundName); }
                function dismiss() { setToast(0); }
                function testNotification() {
                    if (prefs.notifyMuteAll) { setToast(0); setTestMuted(true); return; }
                    setTestMuted(false);
                    setToast(Date.now());
                    if (prefs.notifySound) window.UpurnakaNotificationSound?.play(prefs.notifySoundName);
                }
                pp.effect(() => {
                    if (!toast) return;
                    const timer = setTimeout(() => setToast(0), Number(prefs.notifyDismissAfter) * 1000);
                    return () => clearTimeout(timer);
                }, [toast, prefs.notifyDismissAfter]);
                pp.effect(() => { if (!prefs.notifyMuteAll) setTestMuted(false); }, [prefs.notifyMuteAll]);
                // A notification that arrives or stays up while the tab is in
                // the background flashes its title until the tab is visible.
                const [flashing, setFlashing] = pp.state(false);
                pp.effect(() => {
                    if (!toast || !prefs.notifyFlashTab) return;
                    const check = () => { if (document.hidden) setFlashing(true); };
                    check();
                    document.addEventListener('visibilitychange', check);
                    return () => document.removeEventListener('visibilitychange', check);
                }, [toast, prefs.notifyFlashTab]);
                pp.effect(() => {
                    if (!flashing) return;
                    const title = document.title;
                    let on = false;
                    const timer = setInterval(() => { on = !on; document.title = on ? 'New notification · Upurnaka' : title; }, 1000);
                    const back = () => { if (!document.hidden) setFlashing(false); };
                    document.addEventListener('visibilitychange', back);
                    return () => { clearInterval(timer); document.removeEventListener('visibilitychange', back); document.title = title; };
                }, [flashing]);
            "##</script>
        </div>
    }
}
