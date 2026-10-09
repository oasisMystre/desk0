import 'dart:math' show Point;

import 'package:flutter/gestures.dart';
import 'package:flutter/cupertino.dart';

import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../ffi/types/rdp.dart' show SseMouseButton, SseInputEvent;

class MouseListener extends StatefulWidget {
  final Point scale;
  final Widget child;
  final MouseCursor cursor;
  final ViewerHandle viewer;

  const MouseListener({
    super.key,
    required this.viewer,
    required this.child,
    required this.scale,
    this.cursor = SystemMouseCursors.none,
  });

  @override
  MouseListenerState createState() {
    return MouseListenerState();
  }
}

class MouseListenerState extends State<MouseListener> {
  Offset? pendingMove;
  int lastButtons = 0;
  double accX = 0, accY = 0;
  bool flushScheduled = false;

  static const double kTrackpadSpeed = 1.0;

  static const Map<int, SseMouseButton> buttonMap = {
    kPrimaryButton: SseMouseButton.left,
    kMiddleMouseButton: SseMouseButton.middle,
    kSecondaryButton: SseMouseButton.right,
    kBackMouseButton: SseMouseButton.x1,
    kForwardMouseButton: SseMouseButton.x2,
  };

  Future<void> send(List<SseInputEvent> events) async {
    if (events.isEmpty) return;

    await widget.viewer.sendInputEvents(events: events);
  }

  SseInputEvent positionEvent(Offset local) {
    final dx = local.dx * widget.scale.x;
    final dy = local.dy * widget.scale.y;

    return SseInputEvent.mousePosition(x: dx.round(), y: dy.round());
  }

  void queueMove(Offset local) {
    pendingMove = local;
    if (flushScheduled) return;
    flushScheduled = true;
    WidgetsBinding.instance.scheduleFrameCallback((_) {
      flushScheduled = false;
      final position = pendingMove;
      pendingMove = null;
      if (position != null) send([positionEvent(position)]);
    });
  }

  List<SseInputEvent> buttonEvents(int mask, bool pressed) {
    final List<SseInputEvent> events = [];
    buttonMap.forEach((flag, button) {
      if (mask & flag != 0) {
        events.add(
          pressed
              ? SseInputEvent.mouseButtonPressed(button)
              : SseInputEvent.mouseButtonReleased(button),
        );
      }
    });

    return events;
  }

  void onPointerDown(PointerDownEvent event) {
    Focus.of(context).requestFocus();

    pendingMove = null;
    final newPressed = event.buttons & ~lastButtons;
    lastButtons = event.buttons;
    send([
      positionEvent(event.localPosition),
      ...buttonEvents(newPressed, true),
    ]);
  }

  void onPointerUp(PointerUpEvent event) {
    pendingMove = null;
    final released = lastButtons & ~event.buttons;
    lastButtons = event.buttons;
    send([
      positionEvent(event.localPosition),
      ...buttonEvents(released, false),
    ]);
  }

  void onPointerCancel(PointerCancelEvent event) {
    final held = lastButtons;
    lastButtons = 0;
    send(buttonEvents(held, false));
  }

  void sendWheel(double dx, double dy) {
    accX += dx / 100 * 120;
    accY += dy / 100 * 120;

    final x = accX.truncate();
    final y = accY.truncate();

    accX -= x;
    accY -= y;

    final events = <SseInputEvent>[
      if (y != 0)
        SseInputEvent.wheelRotations(isVertical: true, rotationUnits: y),
      if (x != 0)
        SseInputEvent.wheelRotations(isVertical: false, rotationUnits: x),
    ];
    if (events.isNotEmpty) send(events);
  }

  void onScroll(PointerSignalEvent event) {
    if (event is! PointerScrollEvent) return;

    sendWheel(event.scrollDelta.dx, event.scrollDelta.dy);
  }

  void onPanZoomStart(PointerPanZoomStartEvent event) {
    accX = 0;
    accY = 0;
  }

  void onPanZoomUpdate(PointerPanZoomUpdateEvent event) {
    sendWheel(
      -event.panDelta.dx * kTrackpadSpeed,
      -event.panDelta.dy * kTrackpadSpeed,
    );
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: widget.cursor,
      child: Listener(
        behavior: HitTestBehavior.opaque,
        onPointerHover: (event) => queueMove(event.localPosition),
        onPointerMove: (event) => queueMove(event.localPosition),
        onPointerSignal: onScroll,
        onPointerUp: onPointerUp,
        onPointerDown: onPointerDown,
        onPointerPanZoomStart: onPanZoomStart,
        onPointerPanZoomUpdate: onPanZoomUpdate,
        child: widget.child,
      ),
    );
  }
}
