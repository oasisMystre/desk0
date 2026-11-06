import 'package:get/get.dart';
import 'package:multiview_desktop/multiview_desktop.dart';
import 'package:flutter/cupertino.dart' hide ConnectionState;

import 'viewer/index.dart' show DeviceViewer;
import 'terminal/index.dart' show TerminalViewer;
import '../../components/index.dart' show TabButton;
import '../../channels/channel.dart' show Channel, TrafficLight;
import '../../states/index.dart' show Connection, ConnectionState;

enum ConnectionType { terminal, desktop }

class OpenConnection extends StatefulWidget {
  const OpenConnection({super.key});

  @override
  State<OpenConnection> createState() => OpenConnectionState();
}

class OpenConnectionState extends State<OpenConnection> {
  int currentTabIndex = 0;
  
  late final ConnectionState connectionState;
  late final Future<TrafficLight> trafficLightFrame;

  @override
  void initState() {
    super.initState();
    connectionState = Get.find<ConnectionState>();
    trafficLightFrame = Channel.getTrafficLightFrame();
  }

  void removeConnection(Connection connection) {
    final index = connectionState.removeConnection(connection);
    if (index == 0) return;

    final length = connectionState.connections.length;
    if (length == 0) {
      MultiViewDesktop.of(context).closeWindow();
      return;
    }

    setState(() {
      if (currentTabIndex > index) {
        currentTabIndex--;
      } else if (currentTabIndex >= length) {
        currentTabIndex = length - 1;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: trafficLightFrame,
      builder: (context, snapshot) {
        final double trafficLightWidth = snapshot.hasData
            ? snapshot.data!.width
            : 14.0;

        return SafeArea(
          child: Obx(() {
            final connections = connectionState.connections;

            if (connections.isEmpty) {
              return const Center(child: CupertinoActivityIndicator());
            }

            return Column(
              children: [
                Container(
                  padding: EdgeInsetsGeometry.symmetric(vertical: 4),
                  child: Row(
                    children: [
                      SizedBox(width: trafficLightWidth + 64),
                      for (final (index, connection) in connections.indexed)
                        TabButton(
                          data: connection.device.name,
                          selected: currentTabIndex == index,
                          onClose: () => removeConnection(connection),
                          onPressed: () =>
                              setState(() => currentTabIndex = index),
                        ),
                    ],
                  ),
                ),
                Expanded(
                  child: IndexedStack(
                    index: currentTabIndex,
                    children: [
                      for (final connection in connections)
                        switch (connection.connectionType) {
                          ConnectionType.terminal => TerminalViewer(
                            viewer: connection.viewer,
                            key: ValueKey(connection.device.id),
                          ),
                          ConnectionType.desktop => DeviceViewer(
                            viewer: connection.viewer,
                            size: connection.desktopSize,
                            key: ValueKey(connection.device.id),
                          ),
                        },
                    ],
                  ),
                ),
              ],
            );
          }),
        );
      },
    );
  }
}
