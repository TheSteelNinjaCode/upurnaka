import "/js/pp-reactive-v2.min.js";
// The application's client code, compiled ahead of time. PulsePoint runs
// nothing else, so it is registered before the first mount.
import "/_rahti/client.js";
import { twMerge } from "/js/tailwind-merge.mjs";

const pp = (globalThis).pp;

globalThis.twMerge = twMerge;

// Same-root navigation replaces the body rather than the document. The
// server carries the resolved managed head in a body template so every SEO
// field follows the page, not only its title.
function syncMetadata() {
  const source = document.getElementById("rahti-metadata");
  if (!(source instanceof HTMLTemplateElement)) return;

  document.head
    .querySelectorAll("[data-rahti-metadata]")
    .forEach((element) => element.remove());
  document.head.append(source.content.cloneNode(true));
}

document.addEventListener("pp:navigation:complete", syncMetadata);

if (document.readyState !== "loading") {
  pp?.mount?.();
} else {
  document.addEventListener(
    "DOMContentLoaded",
    () => pp?.mount?.(),
    { once: true },
  );
}
