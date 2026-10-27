use crate::rahti::{Html, component, html};

/// Publishes `window.UpurnakaNotificationSound.play(name)`, a short synthesized
/// alert shared by settings previews and meeting join requests.
#[component]
pub fn NotificationSound() -> Html {
    html! {
        <span class="contents" hidden>
            <script>r##"
                window.UpurnakaNotificationSound ||= (() => {
                    const tones = {
                        default: [[784, 0], [1175, 0.12]],
                        chime: [[1047, 0], [1319, 0.1], [1568, 0.2]],
                        ping: [[1760, 0]],
                        pop: [[523, 0], [392, 0.07]]
                    };
                    let context = null;
                    return {
                        names: Object.keys(tones),
                        play(name) {
                            if (typeof AudioContext !== 'function') return false;
                            try {
                                context ||= new AudioContext();
                                if (context.state === 'suspended') context.resume();
                                const start = context.currentTime + 0.01;
                                for (const [frequency, offset] of tones[name] || tones.default) {
                                    const oscillator = context.createOscillator();
                                    const gain = context.createGain();
                                    oscillator.type = 'sine';
                                    oscillator.frequency.value = frequency;
                                    gain.gain.setValueAtTime(0.0001, start + offset);
                                    gain.gain.exponentialRampToValueAtTime(0.18, start + offset + 0.015);
                                    gain.gain.exponentialRampToValueAtTime(0.0001, start + offset + 0.32);
                                    oscillator.connect(gain).connect(context.destination);
                                    oscillator.start(start + offset);
                                    oscillator.stop(start + offset + 0.34);
                                }
                                return true;
                            } catch (_) { return false; }
                        }
                    };
                })();
            "##</script>
        </span>
    }
}
