# Chat GIFs

Set `GIPHY_API_KEY` in the ignored `.env` file to a dedicated GIPHY Web API key
from https://developers.giphy.com/dashboard/. Restart the development server
after changing environment configuration. Never commit a real key.

This is a public browser integration key: GIPHY requires Trending and Search
requests to originate in the browser. It is deliberately included in the
authenticated chat page; do not reuse a server credential here.

The picker uses the official Trending and Search APIs, a `g` content rating,
350 ms search debounce, cancellation, a ten-second timeout, and explicit Load
more pagination. Search terms go directly to GIPHY. Results keep provider order.
The official Powered by GIPHY mark and API terms link remain visible beneath
the GIF results. The bundled mark comes from GIPHY's attribution asset pack:
https://media.giphy.com/giphy-attribution-marks.zip.

Choosing a GIF attaches it to the current draft. Removing it or switching chats
does not erase another draft. Sending transmits only the GIPHY identifier; the
server validates it and constructs the media URL. GIFs share the existing chat
limitation: messages and drafts live in the page, and new conversations are
previews rather than delivery to other users.

Without a key, the picker shows an unavailable state. A production key and its
limits are managed in the GIPHY developer dashboard.

API reference: https://developers.giphy.com/docs/api/endpoint/.
