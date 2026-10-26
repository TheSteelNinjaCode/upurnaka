let workerPolicy;

function checkedWorkerURL(value) {
  const url = new URL(value, location.origin);
  if (url.origin !== location.origin || !['/js/vendor/background/worker.js', '/js/vendor/background/timer.js'].includes(url.pathname)) {
    throw new TypeError('Only the bundled background worker can be loaded.');
  }
  return url.href;
}

export function createMeetingWorker(url, name) {
  const checked = checkedWorkerURL(url);
  if (globalThis.trustedTypes && !workerPolicy) {
    workerPolicy = trustedTypes.createPolicy('meeting-worker', {
      createScriptURL: checkedWorkerURL,
    });
  }
  return new Worker(workerPolicy ? workerPolicy.createScriptURL(checked) : checked, { name });
}

export async function loadWorker(url, name) { return createMeetingWorker(url, name); }
