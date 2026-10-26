# Meeting background processing assets

`worker.js`, `_cwt-wasm.wasm`, `_cwt-wasm-simd.wasm`, and
`selfie_segmentation_landscape.tflite` are upstream Amazon Chime SDK
background-filter assets for the SDK 3.32 asset group. They are served
unchanged from this application so cameras and backgrounds can be processed
locally without downloading executable code from a third-party origin.

Refresh them from `tools/` with `node vendor-background-assets.mjs`.
`timer.js` is the application-owned frame scheduler.

Upstream integration documentation:
https://aws.github.io/amazon-chime-sdk-js/modules/backgroundfilter_video_processor.html
