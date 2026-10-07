import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart' show TextButton, IconButton;

import 'terminal_tab.dart' show TerminalTab;
import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../states/index.dart' show TerminalStaticChannelState;

class TerminalViewer extends StatefulWidget {
  final ViewerHandle viewer;

  const TerminalViewer({super.key, required this.viewer});

  @override
  TerminalViewerState createState() {
    return TerminalViewerState();
  }
}

class TerminalViewerState extends State<TerminalViewer> {
  late final TerminalStaticChannelState terminalStaticChannelState;
  final List<int> terminalIds = [];

  @override
  void initState() {
    super.initState();

    terminalStaticChannelState = TerminalStaticChannelState(widget.viewer);
  }

  @override
  void dispose() {
    terminalStaticChannelState.dispose();
    super.dispose();
  }

  void createTerminal() {
    final id = terminalIds.length;
    terminalIds.add(id);

    terminalStaticChannelState.create(id);

    setState(() {});
  }

  void removeTerminal(int id) {
    terminalIds.remove(id);
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          children: [
            for (final id in terminalIds)
              TextButton(onPressed: () {}, child: Text('Terminal ${id + 1}')),
            IconButton(
              onPressed: createTerminal,
              icon: const Icon(CupertinoIcons.add),
            ),
          ],
        ),
        Expanded(
          child: IndexedStack(
            index: 0,
            children: [
              for (final id in terminalIds)
                TerminalTab(
                  terminalId: id,
                  terminalStaticChannelState: terminalStaticChannelState,
                ),
            ],
          ),
        ),
      ],
    );
  }
}
