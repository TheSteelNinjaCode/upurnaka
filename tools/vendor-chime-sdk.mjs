// Rebuild public/js/vendor/amazon-chime-sdk.js.
//
//     cd tools && npm install && node vendor-chime-sdk.mjs
//
// Run it when the pinned version in package.json changes, and commit the
// output. This is a vendoring step, not a build step: nothing in the ordinary
// `cargo run` loop touches it, and a clone with no Node installed still
// builds and runs the application. `public/js/pp-reactive-v2.min.js` is
// shipped the same way.
//
// ## Why vendor rather than load from a CDN
//
// amazon-chime-sdk-js publishes CommonJS only — there is no browser bundle on
// unpkg or jsdelivr — so the alternatives were a transform service such as
// esm.sh, or this. A video call is a bad place to discover that a third-party
// service is slow, rate-limiting, or has changed how it resolves a transitive
// dependency, and self-hosting also keeps the Content-Security-Policy at
// `script-src 'self'`: nothing can inject a script from another origin. The
// Background segmentation also uses the local WASM/model assets fetched by
// vendor-background-assets.mjs. src/browser_policy.rs permits WASM and the
// narrowly scoped meeting-worker Trusted Types policy; script sources stay
// same-origin. The two worker adapters below avoid blob-generated code.
//
// ## Why the package index is the entry point
//
// Importing only the half-dozen modules a meeting needs looks tidier and does
// not work: build/task/PromoteToPrimaryMeetingTask.js imports the package
// index anyway, so the index lands in the graph as a *cycle* rather than as
// the root, and the bundle throws "Class extends value undefined" on load — a
// base class read before its module finished evaluating. Entering through the
// index gives the package the evaluation order it was written for.
//
// The messaging session and its SigV4 signer are pruned instead. Chime's
// messaging product is a different service from Chime SDK Meetings and this
// application does not use it; those two files are also the only ones in the
// SDK that reference @aws-sdk/@smithy, whose Node builds (node:https,
// node:fs) cannot be bundled for a browser at all.
//
// ## Why the exports are listed by name
//
// `export * from` a CommonJS package produces an empty module: the names are
// decided at runtime by `exports.X = …`, so there is nothing for a bundler to
// re-export statically. A default import is no good either — the package is
// compiled TypeScript and sets `__esModule`, so interop points `default` at
// `exports.default`, which does not exist. The namespace import is built at
// runtime from the real exports object, and naming what the application uses
// off it is the form that works.
import { build } from 'esbuild';
import { fileURLToPath } from 'node:url';
import { readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
// The SDK normally turns this bundled code into a blob Worker. Serve the
// exact same code as a local file so Trusted Types can validate its URL.
const encodedWorker = require('amazon-chime-sdk-js/build/encodedtransformworkercode/EncodedTransformWorkerCode.js').default;
await writeFile(new URL('../public/js/vendor/background/encoded-transform.js', import.meta.url), encodedWorker);

/// What src/app/meet/j/[id]/page.rs imports. Add a name here when the page
/// starts using it.
const EXPORTS = [
  'ConsoleLogger',
  'LogLevel',
  'DefaultDeviceController',
  'DefaultMeetingSession',
  'MeetingSessionConfiguration',
  'MeetingSessionStatusCode',
  'DefaultVideoTransformDevice',
  'BackgroundBlurVideoFrameProcessor',
  'BackgroundReplacementVideoFrameProcessor',
];

const entry = [
  "import * as ChimeSDK from 'amazon-chime-sdk-js';",
  ...EXPORTS.map((name) => `export const ${name} = ChimeSDK.${name};`),
].join('\n');

const dropMessaging = {
  name: 'drop-chime-messaging',
  setup(pluginBuild) {
    pluginBuild.onLoad({ filter: /encodedtransformmanager[\\/]DefaultEncodedTransformWorkerManager\.js$/ }, async ({ path }) => {
      const original = await readFile(path, 'utf8');
      const blob = 'URL.createObjectURL(new Blob([EncodedTransformWorkerCode_1.default], { type: \'application/javascript\' }))';
      if (!original.includes(blob) || !original.includes('new Worker(this.workerURL)')) {
        throw new Error('Chime media worker source changed: review the Trusted Types adapter.');
      }
      const contents = original
        .replace(blob, 'new URL("/js/vendor/background/encoded-transform.js", location.origin).href')
        .replace('new Worker(this.workerURL)', 'createMeetingWorker(this.workerURL, "MeetingMedia")');
      return { contents: `const { createMeetingWorker } = require(${JSON.stringify(fileURLToPath(new URL('./chime-worker-loader.js', import.meta.url)))});\n` + contents, loader: 'js' };
    });
    // All background assets are self-hosted. This loader grants Trusted
    // Script URLs only for the one bundled segmentation worker.
    pluginBuild.onResolve({ filter: /voicefocus[\\/]loader$/ }, () => ({
      path: fileURLToPath(new URL('./chime-worker-loader.js', import.meta.url)),
    }));
    pluginBuild.onResolve({ filter: /DefaultVideoFrameProcessorTimer$/ }, () => ({
      path: fileURLToPath(new URL('./chime-frame-timer.js', import.meta.url)),
    }));
    pluginBuild.onResolve(
      { filter: /(messagingsession[\\/]DefaultMessagingSession|sigv4[\\/]DefaultSigV4)$/ },
      () => ({ path: 'chime-messaging-stub', namespace: 'chime-stub' }),
    );
    pluginBuild.onLoad({ filter: /.*/, namespace: 'chime-stub' }, () => ({
      contents: 'export default undefined;',
      loader: 'js',
    }));
  },
};

const result = await build({
  stdin: { contents: entry, resolveDir: import.meta.dirname, sourcefile: 'entry.js' },
  bundle: true,
  // An ES module, loaded with `await import(...)` from inside the component's
  // `pp.effect` — the arrangement docs/conventions/pulsepoint-web-apis.md
  // uses for a vendored library. A megabyte then loads when somebody opens a
  // meeting, not on every page of the application.
  format: 'esm',
  target: 'es2020',
  platform: 'browser',
  minify: true,
  legalComments: 'none',
  // protobufjs, which carries Chime's signaling protocol, reaches for Node's
  // `global`. A browser has `globalThis` and nothing else, and the reference
  // is evaluated while the meeting session is being constructed — so without
  // this the SDK loads fine and then throws the moment somebody joins.
  define: { global: 'globalThis' },
  plugins: [dropMessaging],
  outfile: '../public/js/vendor/amazon-chime-sdk.js',
  metafile: true,
});

const leaked = Object.keys(result.metafile.inputs).filter((path) =>
  /@smithy|@aws-sdk/.test(path),
);

if (leaked.length > 0) {
  console.error(
    `The AWS SDK is back in the bundle (${leaked.length} files). Something new ` +
      `imports the messaging session or the SigV4 signer; widen the plugin filter.`,
  );
  process.exit(1);
}

console.log('Wrote ../public/js/vendor/amazon-chime-sdk.js with no AWS SDK in it.');
