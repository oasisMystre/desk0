import 'dart:convert';

import 'package:get/get.dart';

import '../env.dart' show Env;
import '../models/index.dart' show Device;
import '../../ffi/flutter.dart' show ViewerHandle;
import '../ffi/types/rdp.dart' show SseDesktopSize;
import '../features/connection/index.dart' show ConnectionType;

class Connection {
  final Device device;
  final String token, endpoint;
  final SseDesktopSize desktopSize;
  final ConnectionType connectionType;

  late final ViewerHandle viewer = ViewerHandle(
    token: token,
    size: desktopSize,
    endpoint: endpoint,
  );

  Connection({
    required this.device,
    required this.token,
    required this.endpoint,
    required this.desktopSize,
    required this.connectionType,
  });

  factory Connection.fromEncodedString(String encoded) {
    final json =
        jsonDecode(Uri.decodeComponent(encoded)) as Map<String, dynamic>;

    return switch (json) {
      {
        'width': int? width,
        'height': int? height,
        'token': String token,
        'endpoint': String? endpoint,
        'device': Map<String, dynamic> device,
        'connectionType': String connectionType,
      } =>
        Connection(
          token: token,
          device: Device.fromJson(device),
          endpoint: endpoint == null ? Env.rdpEndpoint : endpoint,
          desktopSize: SseDesktopSize(
            width: width == null ? 720 : width,
            height: height == null ? 1280 : height,
          ),
          connectionType: ConnectionType.values.firstWhere(
            (value) => value.name == connectionType,
            orElse: () => ConnectionType.desktop,
          ),
        ),
      _ => throw const FormatException("Failed to deserialize viewer config"),
    };
  }
}

class ConnectionState extends GetxController {
  final RxInt activeIndex = 0.obs;
  final RxList<Connection> connections = <Connection>[].obs;

  void addConnection(Connection connection) {
    final existingIndex = connections.indexWhere(
      (value) => value.device.id == connection.device.id,
    );

    if (existingIndex >= 0) {
      activeIndex.value = existingIndex;
      return;
    }

    connections.add(connection);
    activeIndex.value = connections.length - 1;
  }

  int removeConnection(Connection connection) {
    final index = connections.indexWhere(
      (value) => value.device.id == connection.device.id,
    );

    if (index >= 0) {
      connections.removeAt(index);
      if (activeIndex.value > index) {
        activeIndex.value--;
      } else if (activeIndex.value >= connections.length) {
        activeIndex.value = connections.isNotEmpty ? connections.length - 1 : 0;
      }
    }

    return index;
  }
}
