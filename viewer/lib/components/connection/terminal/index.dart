import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart' show IconButton;

import 'terminal_tab.dart' show TerminalTab;
import '../../../channels/channel.dart' show Channel;
import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../components/tab_button.dart' show TabButton;
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
  int id = 0;
  int currentTabIndex = 0;

  final List<int> terminalIds = [];
  late final TerminalStaticChannelState terminalStaticChannelState;

  @override
  void initState() {
    super.initState();
    widget.viewer.connect();

    terminalStaticChannelState = TerminalStaticChannelState(widget.viewer);
    createTerminal();
  }

  void createTerminal() {
    final terminalId = id++;
    terminalIds.add(terminalId);

    terminalStaticChannelState.create(terminalId);

    setState(() {
      currentTabIndex = terminalIds.length - 1;
    });
  }

  void removeTerminal(int index) {
    if (terminalIds.length == 1) return;

    terminalIds.removeAt(index);
    if (currentTabIndex > index) {
      currentTabIndex--;
    } else if (currentTabIndex == index) {
      if (currentTabIndex >= terminalIds.length) {
        currentTabIndex = terminalIds.length - 1;
      }
    }

    setState(() {});
  }

  @override
  void dispose() {
    terminalStaticChannelState.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: Channel.getTrafficLightFrame(),
      builder: (context, snapshot) {
        final trafficLightWidth = snapshot.hasData ? snapshot.data!.width : 14;

        return SafeArea(
          child: Column(
            children: [
              Container(
                padding: EdgeInsetsGeometry.symmetric(vertical: 4),
                child: Row(
                  children: [
                    SizedBox(width: trafficLightWidth + 64),
                    for (final entry in terminalIds.asMap().entries)
                      TabButton(
                        selected: currentTabIndex == entry.key,
                        onPressed: () {
                          setState(() {
                            currentTabIndex = entry.key;
                          });
                        },
                        onClose: () {
                          removeTerminal(entry.key);
                          terminalStaticChannelState.exit(entry.value);
                        },
                        data: 'Terminal ${entry.key + 1}',
                      ),
                    IconButton(
                      onPressed: createTerminal,
                      icon: const Icon(CupertinoIcons.add),
                    ),
                  ],
                ),
              ),
              Expanded(
                child: IndexedStack(
                  index: currentTabIndex,
                  children: [
                    for (final id in terminalIds)
                      TerminalTab(
                        key: ValueKey(id),
                        terminalId: id,
                        terminalStaticChannelState: terminalStaticChannelState,
                      ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}
