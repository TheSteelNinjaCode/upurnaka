use crate::{
    components::icon::Icon,
    rahti::{Bindings, Html, component, html},
};

#[component]
pub fn DeviceSettings(bindings: Bindings) -> Html {
    html! {
        <div class="device-settings" @{..bindings}>
            <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
                <p class="text-xs text-muted-foreground">
                    "Device choices apply to new meetings in this browser."
                </p>
                <button type="button" class="settings-secondary" onclick={refreshDevices()}>
                    "Refresh devices"
                </button>
            </div>
            <p class="mb-4 text-sm text-danger-ink" role="alert" hidden={!deviceError}>
                {deviceError}
            </p>
            <h3 class="mb-5 text-sm font-semibold">"Audio"</h3>
            <section class="settings-card">
                <div class="settings-card-heading">
                    <Icon name="speaker" class="size-5" />
                    <div>
                        <h3>"Audio settings"</h3>
                        <p>"Select your speaker and microphone."</p>
                    </div>
                </div>
                <label class="settings-row">
                    <span>"Speaker"</span>
                    <select
                        class="settings-select"
                        aria-label="Speaker"
                        value={prefs.speakerId}
                        disabled={!outputSupported}
                        onchange={settings.update("speakerId", target.value)}
                    >
                        <option value="">"System default"</option>
                        <template pp-for="device in speakers">
                            <option key={device.deviceId} value={device.deviceId}>
                                {device.label}
                            </option>
                        </template>
                    </select>
                </label>
                <div class="settings-row">
                    <span class="text-xs text-muted-foreground">
                        {outputSupported ? "Play a sound through the selected speaker." : "Speaker selection is managed by your browser or computer."}
                    </span>
                    <button
                        class="settings-secondary"
                        type="button"
                        disabled={playingTone}
                        onclick={testSpeaker()}
                    >
                        {playingTone ? "Playing…" : "Test speaker"}
                    </button>
                </div>
                <label class="settings-row">
                    <span>"Microphone"</span>
                    <select
                        class="settings-select"
                        aria-label="Microphone"
                        value={prefs.microphoneId}
                        onchange={settings.update("microphoneId", target.value)}
                    >
                        <option value="">"System default"</option>
                        <template pp-for="device in microphones">
                            <option key={device.deviceId} value={device.deviceId}>
                                {device.label}
                            </option>
                        </template>
                    </select>
                </label>
                <div class="settings-row">
                    <span>
                        "Microphone test"
                        <small>
                            "Only starts when you choose Test microphone. Audio stays on this device."
                        </small>
                    </span>
                    <button
                        class="settings-secondary"
                        type="button"
                        disabled={!mediaSupported}
                        aria-label={testingMic ? "Stop microphone test" : "Test microphone"}
                        onclick={setTestingMic(!testingMic)}
                    >
                        {testingMic ? "Stop microphone test" : "Test microphone"}
                    </button>
                </div>
                <div class="settings-row" hidden={!testingMic}>
                    <span class="text-xs text-muted-foreground">"Input level"</span>
                    <meter
                        class="device-level"
                        min="0"
                        max="100"
                        value={micLevel}
                        aria-label="Microphone input level"
                    >
                    </meter>
                </div>
                <label class="settings-row">
                    <span>
                        "Automatically adjust mic sensitivity"
                        <small hidden={supported.autoGainControl}>
                            "Not supported by this browser."
                        </small>
                    </span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Automatically adjust mic sensitivity"
                        checked={prefs.autoGainControl}
                        disabled={!supported.autoGainControl}
                        onchange={settings.update("autoGainControl", target.checked)}
                    />
                </label>
                <div class="settings-row">
                    <span>
                        "Sync device buttons"
                        <small>
                            "Headset button synchronization is managed by your browser and operating system."
                        </small>
                    </span>
                    <span class="text-xs text-muted-foreground">"System managed"</span>
                </div>
                <hr class="my-6 border-edge" />
                <label class="settings-row">
                    <span>
                        "Noise suppression"
                        <small>"Reduce background noise picked up by your microphone."</small>
                        <small hidden={supported.noiseSuppression}>
                            "Not supported by this browser."
                        </small>
                    </span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Noise suppression"
                        checked={prefs.noiseSuppression}
                        disabled={!supported.noiseSuppression}
                        onchange={settings.update("noiseSuppression", target.checked)}
                    />
                </label>
            </section>
            <h3 class="mb-5 mt-6 text-sm font-semibold">"Video"</h3>
            <section class="settings-card">
                <div class="settings-card-heading">
                    <Icon name="video" class="size-5" />
                    <div>
                        <h3>"Video settings"</h3>
                        <p>"Choose a camera and check your self-view before a meeting."</p>
                    </div>
                </div>
                <label class="settings-row">
                    <span>"Camera"</span>
                    <select
                        class="settings-select"
                        aria-label="Camera device"
                        value={prefs.cameraId}
                        onchange={settings.update("cameraId", target.value)}
                    >
                        <option value="">"System default"</option>
                        <template pp-for="device in cameras">
                            <option key={device.deviceId} value={device.deviceId}>
                                {device.label}
                            </option>
                        </template>
                    </select>
                </label>
                <div class="device-camera-preview">
                    <video
                        class="h-full w-full object-contain"
                        pp-ref={preview}
                        autoplay
                        muted
                        playsinline
                        pp-style={videoStyle}
                        hidden={!previewing}
                    >
                    </video>
                    <div
                        class="flex h-full flex-col items-center justify-center gap-3 text-muted-foreground"
                        hidden={previewing}
                    >
                        <Icon name="video-off" class="size-9" />
                        <p class="text-xs">"Your camera is off"</p>
                    </div>
                </div>
                <div class="settings-row">
                    <span class="text-xs text-muted-foreground">
                        "Your preview is private and is never uploaded."
                    </span>
                    <button
                        class="settings-secondary"
                        type="button"
                        disabled={!mediaSupported}
                        aria-label={previewing ? "Stop camera preview" : "Preview camera"}
                        onclick={setPreviewing(!previewing)}
                    >
                        {previewing ? "Stop camera preview" : "Preview camera"}
                    </button>
                </div>
                <div class="settings-row">
                    <span>
                        "Automatically adjust camera controls"
                        <small>"Exposure and focus are managed by your camera and browser."</small>
                    </span>
                    <span class="text-xs text-muted-foreground">"System managed"</span>
                </div>
                <label class="settings-row">
                    <span>
                        "Mirror my video"
                        <small>
                            "Changes your own view only. Other people see your video normally."
                        </small>
                    </span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Mirror my video"
                        checked={prefs.mirrorVideo}
                        onchange={settings.update("mirrorVideo", target.checked)}
                    />
                </label>
                <label class="settings-row">
                    <span>
                        "Adjust brightness"
                        <small>"Brighten your camera preview and meeting self-view."</small>
                    </span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Adjust brightness"
                        checked={prefs.videoBrightness}
                        onchange={settings.update("videoBrightness", target.checked)}
                    />
                </label>
                <label class="settings-row">
                    <span>
                        "Soft focus"
                        <small>"Soften your camera preview and meeting self-view."</small>
                    </span>
                    <input
                        class="settings-switch"
                        type="checkbox"
                        role="switch"
                        aria-label="Soft focus"
                        checked={prefs.softFocus}
                        onchange={settings.update("softFocus", target.checked)}
                    />
                </label>
            </section>
            <p class="settings-note" hidden={mediaSupported}>
                "Camera and microphone access requires a supported browser on HTTPS or localhost."
            </p>
            <p class="settings-note">
                "If device names are hidden, start a microphone test or camera preview and allow access. Missing or unplugged devices use the system default in new meetings."
            </p>
            <script>r##"
                const settings = pp.context(window.UpurnakaSettingsContext);
                const prefs = settings.preferences;
                const mediaSupported = !!navigator.mediaDevices?.getUserMedia;
                const supported = navigator.mediaDevices?.getSupportedConstraints?.() || {};
                const outputSupported = typeof window.AudioContext?.prototype.setSinkId === 'function';
                const [microphones, setMicrophones] = pp.state([]);
                const [speakers, setSpeakers] = pp.state([]);
                const [cameras, setCameras] = pp.state([]);
                const [deviceError, setDeviceError] = pp.state('');
                const [testingMic, setTestingMic] = pp.state(false);
                const [micLevel, setMicLevel] = pp.state(0);
                const [previewing, setPreviewing] = pp.state(false);
                const [playingTone, setPlayingTone] = pp.state(false);
                const preview = pp.ref(null);
                const tone = pp.ref(null);
                const alive = pp.ref(true);
                const videoStyle = 'transform:scaleX(' + (prefs.mirrorVideo ? '-1' : '1') + ');filter:brightness(' + (prefs.videoBrightness ? '1.2' : '1') + ') blur(' + (prefs.softFocus ? '0.6px' : '0px') + ')';
                function describeFailure(failure) {
                    if (failure.name === 'NotAllowedError') return 'Access was denied. Allow camera or microphone access in your browser site settings, then try again.';
                    if (failure.name === 'NotFoundError') return 'No matching device was found. Connect a device or choose System default.';
                    if (failure.name === 'OverconstrainedError') return 'The selected device is unavailable. Choose System default and try again.';
                    if (failure.name === 'NotReadableError') return 'This device is busy or unavailable. Close other apps using it and try again.';
                    return 'The device could not be started. Check your browser and computer settings.';
                }
                const refreshDevices = pp.event(async () => {
                    if (!navigator.mediaDevices?.enumerateDevices) return;
                    try {
                        const found = await navigator.mediaDevices.enumerateDevices();
                        if (!alive.current || !pp.props.active) return;
                        const list = kind => found.filter(device => device.kind === kind && device.deviceId && device.deviceId !== 'default').map((device,index) => ({deviceId:device.deviceId,label:device.label || (kind === 'videoinput' ? 'Camera ' : kind === 'audioinput' ? 'Microphone ' : 'Speaker ') + (index + 1)}));
                        setMicrophones(list('audioinput')); setSpeakers(list('audiooutput')); setCameras(list('videoinput'));
                    } catch (_) { if (alive.current) setDeviceError('Devices could not be listed. Check your browser permissions.'); }
                });
                pp.effect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
                pp.effect(() => {
                    if (!pp.props.active) { setTestingMic(false); setPreviewing(false); return; }
                    refreshDevices();
                    navigator.mediaDevices?.addEventListener('devicechange', refreshDevices);
                    return () => { navigator.mediaDevices?.removeEventListener('devicechange', refreshDevices); };
                }, [pp.props.active, refreshDevices]);
                pp.effect(() => {
                    if (!pp.props.active || !previewing || !mediaSupported) return;
                    let cancelled = false;
                    let stream = null;
                    setDeviceError('');
                    navigator.mediaDevices.getUserMedia({audio:false,video:prefs.cameraId ? {deviceId:{exact:prefs.cameraId}} : true}).then(acquired => {
                        if (cancelled) { acquired.getTracks().forEach(track => track.stop()); return; }
                        stream = acquired;
                        if (preview.current) { preview.current.srcObject = stream; preview.current.play().catch(() => {}); }
                        refreshDevices();
                    }).catch(failure => { if (!cancelled) { setDeviceError(describeFailure(failure)); setPreviewing(false); } });
                    return () => { cancelled = true; stream?.getTracks().forEach(track => track.stop()); if (preview.current) preview.current.srcObject = null; };
                }, [pp.props.active, previewing, prefs.cameraId, refreshDevices]);
                pp.effect(() => {
                    if (!pp.props.active || !testingMic || !mediaSupported) return;
                    let cancelled = false;
                    let stream = null;
                    let context = null;
                    let timer = null;
                    setDeviceError('');
                    (async () => {
                        try {
                            const acquired = await navigator.mediaDevices.getUserMedia({video:false,audio:{...(prefs.microphoneId ? {deviceId:{exact:prefs.microphoneId}} : {}),autoGainControl:prefs.autoGainControl,noiseSuppression:prefs.noiseSuppression,echoCancellation:true}});
                            if (cancelled) { acquired.getTracks().forEach(track => track.stop()); return; }
                            stream = acquired; refreshDevices();
                            context = new AudioContext();
                            await context.resume();
                            if (cancelled) return;
                            const analyser = context.createAnalyser(); analyser.fftSize = 256;
                            context.createMediaStreamSource(stream).connect(analyser);
                            const samples = new Uint8Array(analyser.fftSize);
                            timer = setInterval(() => { analyser.getByteTimeDomainData(samples); let sum = 0; for (const sample of samples) sum += ((sample - 128) / 128) ** 2; setMicLevel(Math.min(100, Math.round(Math.sqrt(sum / samples.length) * 250))); }, 100);
                        } catch (failure) { if (!cancelled) { stream?.getTracks().forEach(track => track.stop()); context?.close().catch(() => {}); setDeviceError(describeFailure(failure)); setTestingMic(false); } }
                    })();
                    return () => { cancelled = true; clearInterval(timer); stream?.getTracks().forEach(track => track.stop()); context?.close().catch(() => {}); setMicLevel(0); };
                }, [pp.props.active, testingMic, prefs.microphoneId, prefs.autoGainControl, prefs.noiseSuppression, refreshDevices]);
                async function testSpeaker() {
                    setDeviceError(''); setPlayingTone(true);
                    let context;
                    try {
                        context = new AudioContext(); tone.current = context;
                        if (prefs.speakerId && context.setSinkId) await context.setSinkId(prefs.speakerId);
                        if (!alive.current || !pp.props.active || tone.current !== context) return;
                        await context.resume();
                        if (!alive.current || !pp.props.active || tone.current !== context) return;
                        const oscillator = context.createOscillator(); const gain = context.createGain();
                        oscillator.frequency.value = 440; gain.gain.value = 0.08;
                        oscillator.connect(gain).connect(context.destination);
                        oscillator.start(); oscillator.stop(context.currentTime + 0.5);
                        await new Promise(resolve => { oscillator.onended = resolve; });
                    } catch (_) { if (alive.current) setDeviceError('The speaker test could not play. Check your output device and browser audio settings.'); }
                    finally { if (context?.state !== 'closed') await context?.close().catch(() => {}); if (tone.current === context) tone.current = null; if (alive.current) setPlayingTone(false); }
                }
                pp.effect(() => {
                    if (!pp.props.active) { const context = tone.current; tone.current = null; context?.close().catch(() => {}); setPlayingTone(false); }
                    return () => { const context = tone.current; tone.current = null; context?.close().catch(() => {}); };
                }, [pp.props.active]);
            "##</script>
        </div>
    }
}
