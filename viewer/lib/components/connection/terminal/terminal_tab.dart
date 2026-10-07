import 'dart:async' show Timer;
import 'dart:convert' show utf8;

import 'package:flutter/cupertino.dart';
import 'package:xterm/xterm.dart'
    show Terminal, TerminalController, TerminalView;

import '../../../states/index.dart' show TerminalStaticChannelState;
import '../../../ffi/types.dart'
    show
        SseTerminalOutputEvent,
        SseTerminalOutputEvent_Connected,
        SseTerminalOutputEvent_Data,
        SseTerminalOutputEvent_Exit;

class TerminalTab extends StatefulWidget {
  final int terminalId;
  final TerminalStaticChannelState terminalStaticChannelState;

  const TerminalTab({
    super.key,
    required this.terminalId,
    required this.terminalStaticChannelState,
  });

  @override
  TerminalTabState createState() {
    return TerminalTabState();
  }
}

class TerminalTabState extends State<TerminalTab> {
  final Terminal terminal = Terminal();
  final controller = TerminalController();

  late Shell shell;

  VoidCallback? removeEventListener;

  @override
  void initState() {
    super.initState();

    widget.terminalStaticChannelState.create(widget.terminalId);

    shell = Shell(
      terminalId: widget.terminalId,
      terminalStaticChannelState: widget.terminalStaticChannelState,
    );

    terminal.onOutput = shell.onInput;

    removeEventListener = widget.terminalStaticChannelState.addEventListener(
      widget.terminalId,
      onEvent,
    );
  }

  void onEvent(SseTerminalOutputEvent event) {
    switch (event) {
      case SseTerminalOutputEvent_Connected():
        debugPrint("connected");
        break;
      case SseTerminalOutputEvent_Data(:final data):
        terminal.write(utf8.decode(data, allowMalformed: true));
      case SseTerminalOutputEvent_Exit(:final code):
        terminal.write("\r\n[Process exited $code]\r\n");
    }
  }

  @override
  void dispose() {
    shell.dispose();
    controller.dispose();
    removeEventListener?.call();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return TerminalView(
      terminal,
      autofocus: true,
      backgroundOpacity: 0,
      controller: controller,
      hardwareKeyboardOnly: true,
      keyboardType: TextInputType.multiline,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      // blinkInterval: const Duration(milliseconds: 600),
    );
  }
}

class Shell {
  final int terminalId;
  final TerminalStaticChannelState terminalStaticChannelState;

  Timer? inputTimer;
  String inputBuffer = '';

  Shell({required this.terminalId, required this.terminalStaticChannelState});

  void onInput(String data) {
    inputBuffer += data;

    inputTimer ??= Timer(const Duration(microseconds: 5), flushInput);
  }

  void flushInput() {
    inputTimer?.cancel();
    inputTimer = null;
    if (inputBuffer.isEmpty) {
      return;
    }

    final data = inputBuffer;
    inputBuffer = '';

    terminalStaticChannelState.sendInput(terminalId, data);
  }

  void dispose() {
    flushInput();
  }
}
