import 'package:get/get.dart';
import 'package:multiview_desktop/multiview_desktop.dart';
import 'package:flutter/cupertino.dart' hide ConnectionState;

import 'viewer/index.dart' show DeviceViewer;
import 'terminal/index.dart' show TerminalViewer;
import '../../channels/channel.dart' show Channel, TrafficLight;
import '../../window_manager.dart' show ConnectionWindowManager;
import '../../states/index.dart' show Connection, ConnectionState;
import '../../components/index.dart' show PlatformImageCard, TabButton;

enum ConnectionType { terminal, desktop }

class OpenConnection extends StatefulWidget {
  const OpenConnection({super.key});

  @override
  State<OpenConnection> createState() => OpenConnectionState();
}

class OpenConnectionState extends State<OpenConnection> {
  int? viewId;
  late final ConnectionState connectionState;
  late final Future<TrafficLight> trafficLightFrame;

  @override
  void initState() {
    super.initState();
    connectionState = Get.find<ConnectionState>();
    trafficLightFrame = Channel.getTrafficLightFrame();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    viewId = viewId != null ? viewId : MultiViewDesktop.getIdByContext(context);
    if (viewId != null) {
      ConnectionWindowManager.instance.onWindowOpened(viewId!);
    }
  }

  @override
  void dispose() {
    if (viewId != null) {
      ConnectionWindowManager.instance.onWindowClosed(viewId!);
    }
    super.dispose();
  }

  void removeConnection(Connection connection) {
    final index = connectionState.removeConnection(connection);
    if (index < 0) return;

    final length = connectionState.connections.length;
    if (length == 0) {
      MultiViewDesktop.of(context).closeWindow();
      return;
    }
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
            final activeIndex = connectionState.activeIndex.value.clamp(
              0,
              connections.length - 1,
            );

            return Column(
              children: [
                DragToMoveArea(
                  child: Container(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Row(
                      children: [
                        SizedBox(width: trafficLightWidth + 64),
                        for (final (index, connection) in connections.indexed)
                          TabButton(
                            data: connection.device.name,
                            prefix: SizedBox(
                              width: 16,
                              height: 16,
                              child: PlatformImageCard(
                                os: connection.device.os,
                              ),
                            ),
                            selected: activeIndex == index,
                            onClose: () => removeConnection(connection),
                            onPressed: () =>
                                connectionState.activeIndex.value = index,
                          ),
                      ],
                    ),
                  ),
                ),
                Expanded(
                  child: IndexedStack(
                    index: activeIndex,
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
