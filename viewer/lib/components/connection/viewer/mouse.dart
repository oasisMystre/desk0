import 'dart:math';

import 'package:flutter/cupertino.dart';
import 'package:flutter/gestures.dart';

import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../ffi/types.dart' show InputEvent, MouseButton;

class MouseListener extends StatefulWidget {
  final Point scale;
  final Widget child;
  final ViewerHandle viewer;

  final MouseCursor cursor;

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
  bool flushScheduled = false;

  static const Map<int, MouseButton> buttonMap = {
    kPrimaryButton: MouseButton.left,
    kMiddleMouseButton: MouseButton.middle,
    kSecondaryButton: MouseButton.right,
    kBackMouseButton: MouseButton.x1,
    kForwardMouseButton: MouseButton.x2,
  };

  Future<void> send(List<InputEvent> events) async {
    if (events.isEmpty) return;

    await widget.viewer.sendInputEvents(events: events);
  }

  InputEvent positionEvent(Offset local) {
    final dx = local.dx * widget.scale.x;
    final dy = local.dy * widget.scale.y;

    return InputEvent.mousePosition(x: dx.round(), y: dy.round());
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

  List<InputEvent> buttonEvents(int mask, bool pressed) {
    final List<InputEvent> events = [];
    buttonMap.forEach((flag, button) {
      if (mask & flag != 0) {
        events.add(
          pressed
              ? InputEvent.mouseButtonPressed(button)
              : InputEvent.mouseButtonReleased(button),
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

  void onScroll(PointerSignalEvent event) {
    if (event is! PointerScrollEvent) return;
    final List<InputEvent> events = [];

    if (event.scrollDelta.dy != 0) {
      events.add(
        InputEvent.wheelRotations(
          isVertical: true,
          rotationUnits: (event.scrollDelta.dy / 100 * 120).round(),
        ),
      );
    }

    if (event.scrollDelta.dx != 0) {
      events.add(
        InputEvent.wheelRotations(
          isVertical: false,
          rotationUnits: (event.scrollDelta.dx / 100 * 120).round(),
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: widget.cursor,
      child: Listener(
        behavior: HitTestBehavior.opaque,
        onPointerHover: (event) {
          queueMove(event.localPosition);
        },
        onPointerMove: (event) {
          queueMove(event.localPosition);
        },
        onPointerSignal: onScroll,
        onPointerUp: onPointerUp,
        onPointerDown: onPointerDown,
        child: widget.child,
      ),
    );
  }
}
