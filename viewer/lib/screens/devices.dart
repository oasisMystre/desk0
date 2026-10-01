import 'package:get/get.dart';
import 'package:flutter/material.dart';
import 'package:liquid_glass_widgets/liquid_glass_widgets.dart'
    show GlassToast;

import '../dio/index.dart' show Api;
import '../states/app_state.dart' show AppState, ListType;
import '../components/connection/index.dart'
    show ConnectionCard, ConnectionListItem;

class DevicesScreen extends StatelessWidget {
  const DevicesScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final Api api = Get.find();
    final AppState appState = Get.find();

    return FutureBuilder(
      future: api.device.getDevices(),
      builder: (context, state) {
        if (state.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }

        if (state.hasError) {
          return GlassToast(message: state.error.toString());
        }

        final devices = state.data!;
        return Obx(() {
          final isList = appState.deviceTabListType.value == ListType.list;

          return GridView.builder(
            itemCount: state.data!.length,
            padding: EdgeInsetsGeometry.only(
              top: 64,
              right: 8,
              left: appState.isSideNavOpened.value ? 0 : 8,
            ),
            gridDelegate: SliverGridDelegateWithMaxCrossAxisExtent(
              maxCrossAxisExtent: isList ? 280 : 150,
              mainAxisExtent: isList ? 48 : 72,
              crossAxisSpacing: 8,
              mainAxisSpacing: 8,
            ),
            itemBuilder: (context, index) {
              final device = devices[index];

              if (isList) return ConnectionListItem(device: device);
              return ConnectionCard(key: ValueKey(index), device: device);
            },
          );
        });
      },
    );
  }
}
