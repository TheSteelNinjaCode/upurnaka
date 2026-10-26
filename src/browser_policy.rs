//! Local meeting segmentation needs WebAssembly and one worker policy.
pub(crate) fn configure() {
    let mut headers = rahti::security::SecurityHeaders::default();
    if let Some(csp) = headers.content_security_policy.as_mut() {
        *csp = csp
            .replace(
                "script-src 'self';",
                "script-src 'self' 'wasm-unsafe-eval';",
            )
            .replace(
                "trusted-types pulsepoint",
                "trusted-types pulsepoint meeting-worker",
            );
    }
    rahti::security::configure(headers);
}
