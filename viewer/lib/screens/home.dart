import 'package:get/get.dart';
import 'package:flutter/material.dart';

import '../states/device_state.dart' show DeviceState;
import '../states/app_state.dart' show AppState, ListType;
import '../features/index.dart' show ConnectionCard, ConnectionListItem;

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final AppState appState = Get.find();
    final DeviceState deviceState = Get.find();

    return Obx(() {
      if (deviceState.isLoading.value && deviceState.devices.isEmpty) {
        return const Center(child: CircularProgressIndicator());
      }

      if (deviceState.error != null && deviceState.devices.isEmpty) {
        return Center(
          child: Text('Unable to load devices: ${deviceState.error}'),
        );
      }

      final isList = appState.deviceTabListType.value == ListType.list;
      final devices = deviceState.devices
          .where((device) => device.favorite)
          .toList(growable: false);

      return GridView.builder(
        itemCount: devices.length,
        padding: EdgeInsetsGeometry.only(
          top: 64,
          right: 8,
          left: appState.isSideNavOpened.value ? 0 : 8,
        ),
        gridDelegate: SliverGridDelegateWithMaxCrossAxisExtent(
          mainAxisSpacing: 8,
          crossAxisSpacing: 8,
          mainAxisExtent: isList ? 48 : 72,
          maxCrossAxisExtent: isList ? 280 : 150,
        ),
        itemBuilder: (context, index) {
          final device = devices[index];

          if (isList) return ConnectionListItem(device: device);
          return ConnectionCard(key: ValueKey(device.id), device: device);
        },
      );
    });
  }
}
