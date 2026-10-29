# Chat emoji and GIF picker

Chat uses the same bundled Fluent 3D emoji catalog as community channel icons.
The Emoji tab shows all nine categories; All shows twelve recent or suggested
emoji alongside GIFs. Search matches emoji, names, and keywords across the whole
catalog, with exact matches first. Category buttons scroll the full Emoji list.

Selection inserts Unicode text at the saved composer selection, preserving
ordinary text and rich composition. Arrow keys navigate the grids and category
buttons; Enter in emoji search inserts the first match. Recent emoji are
deduplicated, validated against the catalog, and saved per account in this
browser, up to 24. Emoji artwork is local; its license and source are recorded
in `public/images/fluent-emoji/SOURCE.md`.

GIFs retain their existing GIPHY connection, privacy preference, search,
pagination, attribution, draft selection, and send behavior.
