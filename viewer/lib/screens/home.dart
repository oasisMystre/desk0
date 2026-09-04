import 'package:desk0/components/connection/connection_list_item.dart';
import 'package:get/get.dart';
import 'package:flutter/material.dart';

import '../states/app_state.dart';
import '../dio/models/device.dart';
import '../components/connection/index.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final AppState appState = Get.find();

    return Obx(() {
      final isList = appState.deviceTabListType.value == ListType.list;
      return GridView.builder(
        itemCount: 8,
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
          if (isList) return ConnectionListItem();
          return ConnectionCard(
            key: ValueKey(index),
            device: Device(
              id: "",
              os: "",
              username: "",
              hostname: "",
              organization: "",
              version: "",
              lastLoggedIn: "",
              createdAt: "",
            ),
          );
        },
      );
    });
  }
}
