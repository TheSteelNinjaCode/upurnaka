use crate::{
    components::{
        icon::Icon,
        rahti_ui::dialog::{
            Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader,
            DialogTitle, DialogTrigger,
        },
    },
    rahti::{Html, component, html},
};

#[component]
pub fn PrivacySettings() -> Html {
    let user = crate::auth::current().unwrap_or_default();
    html! {
        <div class="privacy-settings">
            <section class="settings-card privacy-setting">
                <Icon name="contacts" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Manage how people can find you"</h3>
                    <p>"Review the contact information associated with your Upurnaka account."</p>
                </div>
                <Dialog>
                    <DialogTrigger class="settings-secondary whitespace-nowrap">
                        "Manage contact information"
                    </DialogTrigger>
                    <DialogContent class="privacy-contact-dialog sm:max-w-[500px]!">
                        <DialogHeader>
                            <DialogTitle>"Contact information"</DialogTitle>
                            <DialogDescription>
                                "Your account information and profile discovery."
                            </DialogDescription>
                        </DialogHeader>
                        <dl class="space-y-4 text-sm">
                            <div>
                                <dt class="text-muted-foreground">"Display name"</dt>
                                <dd class="mt-1 break-words">@{user.name.as_str()}</dd>
                            </div>
                            <div>
                                <dt class="text-muted-foreground">"Sign-in email"</dt>
                                <dd class="mt-1 break-all">@{user.email.as_str()}</dd>
                            </div>
                            <div>
                                <dt class="text-muted-foreground">"Phone number"</dt>
                                <dd class="mt-1">"No phone number is stored in your account."</dd>
                            </div>
                        </dl>
                        <p class="text-xs leading-relaxed text-muted-foreground">
                            "Public email and phone-number lookup is not available. Workspace administrators can still manage your account. This panel does not change your sign-in email."
                        </p>
                        <DialogFooter>
                            <a class="settings-secondary" href="/account?edit=profile">
                                "Edit profile"
                            </a>
                            <DialogClose class="settings-secondary">"Close"</DialogClose>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </section>
            <section class="settings-card privacy-setting">
                <Icon name="communities" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Manage who can add you to groups"</h3>
                    <p>
                        "Group invitations are not available yet, so there is no group-invitation permission to change."
                    </p>
                </div>
                <select class="settings-select" aria-label="Who can add you to groups" disabled>
                    <option>"Not available yet"</option>
                </select>
            </section>
            <section class="settings-card privacy-setting">
                <Icon name="contacts" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Manage who can call you"</h3>
                    <p>
                        "Incoming direct calls are not available yet. This does not restrict who can use a meeting link."
                    </p>
                </div>
                <select class="settings-select" aria-label="Who can call you" disabled>
                    <option>"Not available yet"</option>
                </select>
            </section>
            <section class="settings-card privacy-setting">
                <Icon name="video-off" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Turn off my camera and mic when my screen locks"</h3>
                    <p>
                        "Stop camera, microphone, screen sharing, and recording when your computer locks. Turn devices on again manually after unlocking."
                    </p>
                    <p hidden={lockSupported}>
                        "Screen-lock detection is not supported in this browser."
                    </p>
                    <p hidden={!lockSupported}>
                        "Requires browser permission. Applies to new meetings."
                    </p>
                </div>
                <label class="accessibility-control">
                    <span aria-hidden="true">
                        {prefs.lockMediaOnScreenLock && lockSupported ? "On" : "Off"}
                    </span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Turn off my camera and mic when my screen locks"
                        checked={prefs.lockMediaOnScreenLock && lockSupported}
                        disabled={!lockSupported || requestingPermission}
                        onchange={changeLockProtection(target.checked)}
                    />
                </label>
            </section>
            <p class="mb-4 text-sm text-danger-ink" role="alert" hidden={!permissionError}>
                {permissionError}
            </p>
            <section class="settings-card privacy-setting">
                <Icon name="list" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Extended diagnostics data"</h3>
                    <p>
                        "Optional extended diagnostic collection is not enabled. Essential operational and development diagnostics remain active."
                    </p>
                </div>
                <label class="accessibility-control">
                    <span aria-hidden="true">"Off"</span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Extended diagnostics data"
                        disabled
                    />
                </label>
            </section>
            <section class="settings-card privacy-setting">
                <Icon name="contacts" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Your profile"</h3>
                    <p>"Manage the name and picture on your Upurnaka profile."</p>
                </div>
                <a href="/account?edit=profile" class="settings-secondary whitespace-nowrap">
                    "Manage profile"
                </a>
            </section>
            <section class="settings-card privacy-setting">
                <Icon name="sparkle" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Optional connected experiences"</h3>
                    <p>
                        "Allow GIPHY searches and GIF images. Turning this off prevents optional GIF requests and hides GIF images in this browser. Meetings and file storage stay available."
                    </p>
                </div>
                <label class="accessibility-control">
                    <span aria-hidden="true">{prefs.connectedExperiences ? "On" : "Off"}</span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Optional connected experiences"
                        checked={prefs.connectedExperiences}
                        onchange={settings.update("connectedExperiences", target.checked)}
                    />
                </label>
            </section>
            <section class="settings-card privacy-setting">
                <Icon name="list" class="size-5" />
                <div class="min-w-0 flex-1">
                    <h3>"Experiences that analyze your content"</h3>
                    <p>
                        "Optional cloud content-analysis features are not available. Local camera effects and audio processing remain controlled by your meeting and device settings."
                    </p>
                </div>
                <label class="accessibility-control">
                    <span aria-hidden="true">"Off"</span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Experiences that analyze your content"
                        disabled
                    />
                </label>
            </section>
            <script>r##"
                const settings = pp.context(window.UpurnakaSettingsContext);
                const prefs = settings.preferences;
                const lockSupported = typeof window.IdleDetector === 'function' && window.isSecureContext;
                const [requestingPermission, setRequestingPermission] = pp.state(false);
                const [permissionError, setPermissionError] = pp.state('');
                async function changeLockProtection(enabled) {
                    setPermissionError('');
                    if (!enabled) { settings.update('lockMediaOnScreenLock', false); return; }
                    if (!lockSupported) return;
                    setRequestingPermission(true);
                    try {
                        const permission = await IdleDetector.requestPermission();
                        if (permission !== 'granted') { setPermissionError('Screen-lock protection was not enabled. Allow idle detection in your browser site settings and try again.'); return; }
                        settings.update('lockMediaOnScreenLock', true);
                    } catch (_) { setPermissionError('Screen-lock protection could not be enabled in this browser.'); }
                    finally { setRequestingPermission(false); }
                }
            "##</script>
        </div>
    }
}
