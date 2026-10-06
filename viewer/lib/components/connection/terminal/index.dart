import 'dart:convert' show utf8;
import 'dart:async' show StreamSubscription;

import 'package:flutter/cupertino.dart';
import 'package:xterm/xterm.dart';

import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../channels/channel.dart' show Channel;
import '../../../ffi/types.dart'
    show
        SseTerminalInput,
        SseStaticChannelOutput,
        SseStaticChannelInput,
        SseTerminalOutput_StdOut,
        SseTerminalOutput_StdErr,
        SseStaticChannelOutput_Terminal;

class TerminalViewer extends StatefulWidget {
  final ViewerHandle viewer;

  const TerminalViewer({super.key, required this.viewer});

  @override
  TerminalViewerState createState() {
    return TerminalViewerState();
  }
}

class TerminalViewerState extends State<TerminalViewer> {
  final Terminal terminal = Terminal();
  final controller = TerminalController();

  StreamSubscription<SseStaticChannelOutput>? subscription;

  @override
  void initState() {
    super.initState();

    terminal.onOutput = (data) {
      // widget.viewer.sendStaticChannelEvents(
      //   events: [
      //     SseStaticChannelInput.terminal(
      //       SseTerminalInput.stdIn(utf8.encode(data)),
      //     ),
      //   ],
      // );
      //
      print(data);
      terminal.write(data);
    };

    widget.viewer.onStaticChannelEventEvent().listen((event) {
      switch (event) {
        case SseStaticChannelOutput_Terminal(:final field0):
          switch (field0) {
            case SseTerminalOutput_StdOut(:final field0):
              terminal.write(utf8.decode(field0));
            case SseTerminalOutput_StdErr(:final field0):
              terminal.write(utf8.decode(field0));
          }
          break;
      }
    });
  }

  @override
  void dispose() {
    controller.dispose();
    subscription?.cancel();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: Channel.getTrafficLightFrame(),
      builder: (context, snapshot) {
        final trafficLightHeight = snapshot.hasData
            ? snapshot.data!.height
            : 14;

        return SafeArea(
          child: Padding(
            padding: EdgeInsets.only(top: trafficLightHeight + 24),
            child: TerminalView(
              terminal,
              autofocus: true,
              backgroundOpacity: 0,
              controller: controller,
              keyboardType: TextInputType.multiline,
              padding: const EdgeInsets.symmetric(horizontal: 12),
              hardwareKeyboardOnly: true,

              // blinkInterval: const Duration(milliseconds: 600),
            ),
          ),
        );
      },
    );
  }
}
