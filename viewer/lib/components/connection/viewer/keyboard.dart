import 'package:flutter/services.dart' show KeyEvent, KeyUpEvent;
import 'package:flutter/cupertino.dart'
    show
        State,
        StatefulWidget,
        Widget,
        FocusNode,
        KeyEventResult,
        BuildContext,
        Focus;

import 'hid_to_scancode.dart' show hidToScancode;
import '../../../ffi/types/rdp.dart' show SseInputEvent;
import '../../../ffi/flutter.dart' show ViewerHandle;

class KeyboardListener extends StatefulWidget {
  final Widget child;
  final ViewerHandle viewer;

  const KeyboardListener({
    super.key,
    required this.viewer,
    required this.child,
  });

  @override
  KeyboardListenerState createState() {
    return KeyboardListenerState();
  }
}

class KeyboardListenerState extends State<KeyboardListener> {
  final focusNode = FocusNode(debugLabel: 'RdpKeyboardListener');

  final Set<int> held = {};

  @override
  void dispose() {
    releaseAll();
    focusNode.dispose();
    super.dispose();
  }

  Future<void> send(List<SseInputEvent> events) async {
    if (events.isEmpty) return;
    widget.viewer.sendInputEvents(events: events);
  }

  KeyEventResult onKey(FocusNode node, KeyEvent event) {
    final usage = event.physicalKey.usbHidUsage;
    if (usage >> 16 != 0x07) return KeyEventResult.ignored;

    final scancode = hidToScancode[usage & 0xFFFF];
    if (scancode == null) return KeyEventResult.ignored;
    final (code, extended) = scancode;
    final id = (extended ? 0x100 : 0) | code;
    if (event is KeyUpEvent) {
      if (held.remove(id)) {
        send([SseInputEvent.keyReleased(code: code, extended: extended)]);
      }
    } else {
      held.add(id);
      send([SseInputEvent.keyPressed(code: code, extended: extended)]);
    }

    return KeyEventResult.handled;
  }

  void releaseAll() {
    if (held.isEmpty) return;
    final events = [
      for (final id in held)
        SseInputEvent.keyReleased(code: id & 0xFF, extended: id & 0x100 != 0),
    ];

    held.clear();
    send(events);
  }

  @override
  Widget build(BuildContext context) {
    return Focus(
      focusNode: focusNode,
      autofocus: true,
      onKeyEvent: onKey,
      onFocusChange: (hasFocus) {
        if (!hasFocus) releaseAll();
      },
      child: widget.child,
    );
  }
}
