// In-process browser test fixture. Never served by the application binary.
export const LogLevel = { WARN: 1 };
export class ConsoleLogger {}
export const MeetingSessionStatusCode = { Left: 0, MeetingEnded: 1 };
export class MeetingSessionConfiguration { constructor(meeting, attendee) { this.attendee = attendee; } }
export class DefaultDeviceController {
  async listVideoInputDevices() { return [{ deviceId: 'test-camera' }, { deviceId: 'second-camera' }]; }
  async startVideoInput(input) {
    this.lastInput = input;
    const canvas = document.createElement('canvas'); canvas.width = 640; canvas.height = 360;
    const paint = canvas.getContext('2d'); paint.fillStyle = '#385c73'; paint.fillRect(0,0,640,360);
    this.stream = canvas.captureStream(15);
    return this.stream;
  }
  async stopVideoInput() { this.stream?.getTracks().forEach(track => track.stop()); }
  startVideoPreviewForVideoInput(video) { video.srcObject = this.stream; }
  stopVideoPreviewForVideoInput(video) { video.srcObject = null; }
}
export class DefaultMeetingSession {
  constructor(configuration, logger, controller) {
    const fixture = { sent: [], stopped: false, muted: false, controller, remoteStreams: new Map() };
    globalThis.roomFixture = fixture;
    const av = {
      addObserver(observer) { fixture.observer = observer; },
      addContentShareObserver(observer) { fixture.shareObserver = observer; },
      realtimeSubscribeToAttendeeIdPresence(callback) { fixture.presence = callback; },
      realtimeUnsubscribeToAttendeeIdPresence() {},
      realtimeSubscribeToReceiveDataMessage(topic, callback) { fixture.message = callback; },
      realtimeUnsubscribeFromReceiveDataMessage() {},
      realtimeSendDataMessage(topic, data) { fixture.sent.push(JSON.parse(data)); },
      async listAudioInputDevices() { return []; },
      async bindAudioElement() {},
      async chooseAudioOutput() {},
      async stopAudioInput() {},
      start() { setTimeout(() => fixture.observer.audioVideoDidStart(), 10); },
      stop() { fixture.stopped = true; fixture.remoteStreams.forEach(stream => stream.getTracks().forEach(track => track.stop())); fixture.observer.audioVideoDidStop({ statusCode: () => 0 }); },
      realtimeMuteLocalAudio() { fixture.muted = true; },
      realtimeUnmuteLocalAudio() { fixture.muted = false; },
      startLocalVideoTile() { fixture.observer.videoTileDidUpdate({ localTile: true, tileId: 1, boundAttendeeId: configuration.attendee.AttendeeId }); },
      stopLocalVideoTile() {},
      bindVideoElement(tile, video) {
        if (tile === 1) { video.srcObject = controller.stream; return; }
        if (!fixture.remoteStreams.has(tile)) {
          const canvas = document.createElement('canvas'); canvas.width = 640; canvas.height = 360;
          const paint = canvas.getContext('2d'); paint.fillStyle = '#385c73'; paint.fillRect(0, 0, 640, 360);
          fixture.remoteStreams.set(tile, canvas.captureStream(15));
        }
        video.srcObject = fixture.remoteStreams.get(tile);
      },
      async startContentShare(stream) { fixture.shared = stream; fixture.shareObserver.contentShareDidStart?.(); },
      stopContentShare() { fixture.shared?.getTracks().forEach(track => track.stop()); fixture.shareObserver.contentShareDidStop?.(); },
    };
    this.audioVideo = av;
  }
}
