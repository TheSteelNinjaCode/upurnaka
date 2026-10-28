# Chat editor

`chat-editor.js` bundles the ProseMirror packages as an ES module. It contains
vendor code only. The application creates and controls the editor inside the
chat page's compiled `html!` script. The editor owns an otherwise empty host;
message HTML is rendered by PulsePoint from a validated text-only wire format.

ProseMirror is MIT licensed; package licenses are in `chat-editor.LICENSE.txt`.
Exact dependency versions are recorded in `chat-editor.lock.json` and the entry
point is `chat-editor.entry.txt`. No Node runtime is needed to run the app.

To reproduce the bundle, in a temporary directory copy the lock file as
`package-lock.json`, copy its root `packages[""].dependencies` into a
`package.json` dependencies object, and copy the entry point as `entry.js`.
Run `npm ci`, followed by:

```text
npx esbuild entry.js --bundle --format=esm --minify --outfile=chat-editor.js
```

Copy the resulting bundle here. When upgrading, update the lock and licenses
and run the chat composer browser tests under the application's normal CSP.
Clipboard input uses plain text so external HTML never enters an HTML parser
or requires a Trusted Types policy.
