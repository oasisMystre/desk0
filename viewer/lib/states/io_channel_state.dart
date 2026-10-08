import 'dart:convert' show utf8;

import '../ffi/flutter.dart' show ViewerHandle;
import '../states/static_channel.dart' show StaticChannel;
import 'terminal_channel_state.dart' show TerminalStaticChannelState;
import '../ffi/types/static_channels.dart' show SseStaticChannelInputEvent;
import '../ffi/types/static_channels/io.dart'
    show
        SseIoInputEvent,
        SseIoOutputEvent,
        SseIoOutputEvent_DownloadFailed,
        SseIoOutputEvent_DownloadStarted,
        SseIoOutputEvent_DownloadProgress,
        SseIoOutputEvent_DownloadCompleted;

class IoStaticChannelState extends StaticChannel<SseIoOutputEvent> {
  int id = 0;
  final ViewerHandle viewer;
  final TerminalStaticChannelState terminal;

  IoStaticChannelState({required this.viewer, required this.terminal}) {
    viewer.onIoOutputEventEvent().listen(onEvent);
  }

  Future<void> download(int id, String uri, String path) async {
    await viewer.sendStaticChannelInputEvents(
      events: [
        SseStaticChannelInputEvent.io(
          SseIoInputEvent.download(
            id: id,
            uri: utf8.encode(uri),
            path: utf8.encode(path),
          ),
        ),
      ],
    );
  }

  int getUniqueId() {
    int id = this.id;
    this.id += 1;
    return id;
  }

  @override
  Future<void> poll() async {
    viewer.sendStaticChannelInputEvents(
      events: [SseStaticChannelInputEvent.io(SseIoInputEvent.poll())],
    );
  }

  @override
  int getIdFromEvent(SseIoOutputEvent event) {
    late int eventId;
    switch (event) {
      case SseIoOutputEvent_DownloadStarted(:final field0):
        eventId = field0;
      case SseIoOutputEvent_DownloadProgress(:final id):
        eventId = id;
      case SseIoOutputEvent_DownloadCompleted(:final id):
        eventId = id;
      case SseIoOutputEvent_DownloadFailed(:final id):
        eventId = id;
    }

    return eventId;
  }
}
