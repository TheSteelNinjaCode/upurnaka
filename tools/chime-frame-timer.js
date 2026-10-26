// Chime's frame timer, using a static same-origin worker instead of blob code.
import { createMeetingWorker } from './chime-worker-loader.js';
export default class MeetingFrameTimer {
  constructor() {
    this.worker = createMeetingWorker('/js/vendor/background/timer.js', 'MeetingFrameTimer');
  }
  async start(delay, callback) {
    this.worker.onmessage = () => callback();
    this.worker.postMessage(delay);
  }
  async destroy() {
    this.worker.onmessage = null;
    this.worker.terminate();
  }
}
