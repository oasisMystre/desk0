import 'dart:async';
import 'dart:convert';

import 'package:get/get.dart' show GetxController;
import 'package:flutter/widgets.dart' show VoidCallback;

import '../../ffi/flutter.dart' show ViewerHandle;
import '../../ffi/types.dart'
    show
        SseTerminalInputEvent,
        SseTerminalOutputEvent,
        SseTerminalOutputEvent_Data,
        SseTerminalOutputEvent_Exit,
        SseTerminalOutputEvent_Connected;

typedef Listener = Function(SseTerminalOutputEvent);

class TerminalStaticChannelState extends GetxController {
  int listenerId = 0;

  Timer? pollTimer;
  StreamSubscription? subscription;

  final ViewerHandle viewer;
  final Map<int, Map<int, Listener>> subscriptions = {};
  final Map<int, List<SseTerminalOutputEvent>> queues = {};

  TerminalStaticChannelState(this.viewer) {
    subscription = viewer.onTerminalOutputEventEvent().listen((event) {
      final id = getIdFromTerminalOutputEvent(event);
      final listeners = subscriptions[id];

      if (listeners == null || listeners.isEmpty) {
        final queue = queues.putIfAbsent(id, () => []);

        if (queue.length >= 64) queue.removeAt(0);
        queue.add(event);
        return;
      }

      for (final listener in listeners.values) {
        listener(event);
      }
    });

    pollTimer = Timer.periodic(
      const Duration(milliseconds: 400),
      (_) => pollForTerminalOutput(),
    );
  }

  int getIdFromTerminalOutputEvent(SseTerminalOutputEvent event) {
    late int terminalId;

    switch (event) {
      case SseTerminalOutputEvent_Connected(:final field0):
        terminalId = field0;
      case SseTerminalOutputEvent_Data(:final id):
        terminalId = id;
      case SseTerminalOutputEvent_Exit(:final id):
        terminalId = id;
    }

    return terminalId;
  }

  @override
  void dispose() {
    super.dispose();

    pollTimer?.cancel();
    subscription?.cancel();
    subscriptions.clear();
  }

  Future<void> create(int id) async {
    await viewer.sendTerminalInputEvents(
      events: [SseTerminalInputEvent.create(id)],
    );
  }

  Future<void> sendInput(int id, String data) async {
    await viewer.sendTerminalInputEvents(
      events: [SseTerminalInputEvent.data(id: id, data: utf8.encode(data))],
    );
  }

  Future<void> resize(
    int id,
    int rows,
    int cols,
    int pixelWidth,
    int pixelHeight,
  ) async {
    await viewer.sendTerminalInputEvents(
      events: [
        SseTerminalInputEvent.resize(
          id: id,
          rows: rows,
          cols: cols,
          pw: pixelWidth,
          ph: pixelWidth,
        ),
      ],
    );
  }

  Future<void> pollForTerminalOutput() async {
    viewer.sendTerminalInputEvents(events: [SseTerminalInputEvent.output()]);
  }

  VoidCallback addEventListener(int terminalId, Listener listener) {
    final listenerId = this.listenerId++;
    subscriptions.putIfAbsent(terminalId, () => {})[listenerId] = listener;
    final queue = queues.remove(terminalId);
    if (queue != null) {
      for (final event in queue) {
        listener(event);
      }
    }

    return () {
      final listeners = subscriptions[terminalId];
      listeners?.remove(listenerId);
      if (listeners != null && listeners.isEmpty) {
        listeners.remove(terminalId);
      }
    };
  }
}
