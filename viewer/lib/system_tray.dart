import 'dart:io';

import 'package:get/get.dart';
import 'package:system_tray/system_tray.dart';
import 'package:url_launcher/url_launcher.dart';

import 'env.dart' show Env;
import 'extensions.dart' show FutureIgnoreError;
import 'states/connection_state.dart' show ConnectionState;
import 'window_manager.dart' show ConnectionWindowManager;

late final SystemTray systemTray;

List<MenuItemBase>? lastItems;
List<MenuItemBase>? lastExtraDefaultItems;

Future<void> initSystemTray() async {
  String iconPath = Platform.isWindows
      ? 'assets/app_icon.ico'
      : 'assets/app_icon.png';

  systemTray = SystemTray();

  await systemTray.initSystemTray(iconPath: iconPath);
  systemTray.registerSystemTrayEventHandler((eventName) {
    final hasConnections =
        Get.isRegistered<ConnectionState>() &&
        Get.find<ConnectionState>().connections.isNotEmpty;

    if (eventName == kSystemTrayEventClick) {
      Platform.isWindows
          ? (hasConnections
                ? ConnectionWindowManager.instance.show()
                : systemTray.popUpContextMenu())
          : systemTray.popUpContextMenu();
    } else if (eventName == kSystemTrayEventRightClick) {
      Platform.isWindows
          ? systemTray.popUpContextMenu()
          : (hasConnections
                ? ConnectionWindowManager.instance.show()
                : systemTray.popUpContextMenu());
    } else if (eventName == kSystemTrayEventDoubleClick) {
      if (hasConnections) {
        ConnectionWindowManager.instance.show();
      }
    }
  });

  if (Get.isRegistered<ConnectionState>()) {
    Get.find<ConnectionState>().connections.listen((_) {
      updateTrayMenu();
    });
  }

  await updateTrayMenu();
}

Future<void> updateTrayMenu({
  List<MenuItemBase>? items,
  List<MenuItemBase>? extraDefaultItems,
}) async {
  if (items != null) lastItems = items;
  if (extraDefaultItems != null) lastExtraDefaultItems = extraDefaultItems;

  final Menu menu = Menu();
  final hasConnections =
      Get.isRegistered<ConnectionState>() &&
      Get.find<ConnectionState>().connections.isNotEmpty;

  await menu.buildFrom([
    ...?lastItems,
    if (lastItems != null) MenuSeparator(),
    ...?lastExtraDefaultItems,
    if (hasConnections) ...[
      MenuItemLabel(
        label: 'Hide',
        onClicked: (item) => ConnectionWindowManager.instance.hide(),
      ),
      MenuItemLabel(
        label: 'Show Desk0',
        onClicked: (item) => ConnectionWindowManager.instance.show(),
      ),
    ] else ...[
      MenuItemLabel(
        label: "Open Desk0",
        onClicked: (item) async {
          final uri = Uri.parse(Env.consoleUrl);
          await launchUrl(uri, mode: LaunchMode.externalApplication);
        },
      ),
    ],
    MenuItemLabel(
      label: 'Quit',
      onClicked: (item) async {
        await systemTray.destroy().ignoreError();
        await ConnectionWindowManager.instance.close(quit: true);
      },
    ),
  ]);

  await systemTray.setContextMenu(menu);
}
