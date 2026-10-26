// Static frame scheduler for the local meeting video processor.
self.onmessage = event => {
  setTimeout(() => self.postMessage(null), event.data);
};
