// Assets used by the pinned Chime SDK's built-in person segmentation.
// Run from tools/: node vendor-background-assets.mjs
import { mkdir, writeFile } from 'node:fs/promises';
const output = new URL('../public/js/vendor/background/', import.meta.url);
await mkdir(output, { recursive: true });
for (const [name, path] of [
  ['worker.js', 'workers/worker.js'],
  ['_cwt-wasm.wasm', 'wasm/_cwt-wasm.wasm'],
  ['_cwt-wasm-simd.wasm', 'wasm/_cwt-wasm-simd.wasm'],
  ['selfie_segmentation_landscape.tflite', 'models/selfie_segmentation_landscape.tflite'],
]) {
  const response = await fetch(`https://static.sdkassets.chime.aws/bgblur/${path}?assetGroup=sdk-3.32`);
  if (!response.ok) throw new Error(`${name}: ${response.status}`);
  const bytes = new Uint8Array(await response.arrayBuffer());
  await writeFile(new URL(name, output), bytes);
  console.log(`${name}: ${bytes.length} bytes`);
}
