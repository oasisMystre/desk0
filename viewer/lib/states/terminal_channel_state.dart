import 'dart:async';
import 'dart:convert';

import 'static_channel.dart' show StaticChannel;
import '../../ffi/flutter.dart' show ViewerHandle;
import '../../ffi/types/static_channels.dart' show SseStaticChannelInputEvent;
import '../../ffi/types/static_channels/terminal.dart'
    show
        SseTerminalInputEvent,
        SseTerminalMode,
        SseTerminalOutputEvent,
        SseTerminalOutputEvent_Data,
        SseTerminalOutputEvent_Exit,
        SseTerminalOutputEvent_Error,
        SseTerminalOutputEvent_Connected;

class TerminalStaticChannelState extends StaticChannel<SseTerminalOutputEvent> {
  final ViewerHandle viewer;

  TerminalStaticChannelState(this.viewer) {
    subscription = viewer.onTerminalOutputEventEvent().listen(onEvent);
  }

  Future<void> create(int id) async {
    await viewer.sendStaticChannelInputEvents(
      events: [
        SseStaticChannelInputEvent.terminal(SseTerminalInputEvent.create(id)),
      ],
    );
  }

  Future<void> send(
    int id,
    String data, {
    SseTerminalMode mode = SseTerminalMode.async_,
  }) async {
    await viewer.sendStaticChannelInputEvents(
      events: [
        SseStaticChannelInputEvent.terminal(
          SseTerminalInputEvent.data(
            id: id,
            mode: mode,
            data: utf8.encode(data),
          ),
        ),
      ],
    );
  }

  Future<String> sendSync(int id, String command) async {
    final completer = Completer<String>();

    late final void Function() unsubscribe;

    void complete(String data) {
      if (!completer.isCompleted) {
        completer.complete(data);
      }
    }

    void fail(Object error) {
      if (!completer.isCompleted) {
        completer.completeError(error);
      }
    }

    String line = '';

    unsubscribe = addEventListener(id, (event) {
      switch (event) {
        case SseTerminalOutputEvent_Data(:final data):
          line += utf8.decode(data, allowMalformed: true);
        case SseTerminalOutputEvent_Exit(:final code, :final description):
          if (code == 0) {
            complete(line);
          } else {
            fail("[Process exited with $code] $description");
          }
          unsubscribe();
        case SseTerminalOutputEvent_Error(:final description, :final code):
          fail("Terminal exited with $code $description");
          unsubscribe();
        case SseTerminalOutputEvent_Connected():
          break;
      }
    });

    await send(id, command, mode: SseTerminalMode.sync_).catchError((error) {
      unsubscribe();
      throw error;
    });

    return completer.future;
  }

  Future<void> exit(int id) async {
    await viewer.sendStaticChannelInputEvents(
      events: [
        SseStaticChannelInputEvent.terminal(SseTerminalInputEvent.exit(id)),
      ],
    );
  }

  Future<void> resize(
    int id,
    int rows,
    int cols,
    int pixelWidth,
    int pixelHeight,
  ) async {
    await viewer.sendStaticChannelInputEvents(
      events: [
        SseStaticChannelInputEvent.terminal(
          SseTerminalInputEvent.resize(
            id: id,
            rows: rows,
            cols: cols,
            pw: pixelWidth,
            ph: pixelWidth,
          ),
        ),
      ],
    );
  }

  @override
  Future<void> poll() async {
    await viewer.sendStaticChannelInputEvents(
      events: [
        SseStaticChannelInputEvent.terminal(SseTerminalInputEvent.poll()),
      ],
    );
  }

  @override
  int getIdFromEvent(SseTerminalOutputEvent event) {
    late int terminalId;

    switch (event) {
      case SseTerminalOutputEvent_Connected(:final field0):
        terminalId = field0;
      case SseTerminalOutputEvent_Data(:final id):
        terminalId = id;
      case SseTerminalOutputEvent_Exit(:final id):
        terminalId = id;
      case SseTerminalOutputEvent_Error(:final id):
        terminalId = id;
    }

    return terminalId;
  }
}
