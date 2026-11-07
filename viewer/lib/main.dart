import 'package:get/get.dart';
import 'package:multiview_desktop/multiview_desktop.dart';
import 'package:flutter/material.dart' hide ConnectionState;

import 'extensions.dart' show DeepLinkUri;
import 'system_tray.dart' show initSystemTray;
import 'theme/app_theme.dart' show AppColors;
import 'ffi/frb_generated.dart' show RustLib;
import 'deep_link_service.dart' show DeepLinkService;
import 'window_manager.dart' show ConnectionWindowManager;
import 'states/index.dart' show ConnectionState, Connection;
import 'l10n/generated/app_localizations.dart' show AppLocalizations;

final DeepLinkService deepLinkService = DeepLinkService();

class ViewerWindowObserver extends WindowObserver {
  @override
  void onWindowClosed(int viewId) {
    ConnectionWindowManager.instance.onWindowClosed(viewId);
  }
}

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();

  Get.put(ConnectionState());

  await initSystemTray();
  runMultiApp(
    home: (context, publicId) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        MultiViewDesktop.fromId(publicId).hide();
      });

      return MaterialApp(
        title: 'Desk0',
        supportedLocales: AppLocalizations.supportedLocales,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        debugShowCheckedModeBanner: false,
        theme: ThemeData(
          useMaterial3: true,
          brightness: Brightness.dark,
          fontFamily: 'MonaSans',
          scaffoldBackgroundColor: Colors.transparent,
          colorScheme: const ColorScheme.dark(
            primary: AppColors.accentPrimary,
            surface: AppColors.mainBackground,
          ),
          iconButtonTheme: const IconButtonThemeData(
            style: ButtonStyle(
              minimumSize: WidgetStatePropertyAll(Size(32, 32)),
              iconSize: WidgetStatePropertyAll(18.0),
              padding: WidgetStatePropertyAll(EdgeInsets.all(0)),
            ),
          ),
        ),
        home: const MainApplication(),
      );
    },
    config: MultiAppConfig(
      observers: [ViewerWindowObserver()],
      generalParams: const MultiPlatformParams(
        enableDynamicAnchor: true,
        closeMode: CloseMode.softCascade,
      ),
      macosParams: MacosPlatformParams(
        closeAppAfterLastWindowClosed: false,
        saveLastWindowToReopen: false,
        onTaskbarTap: () => ConnectionWindowManager.instance.show(),
        onTerminate: () async {
          await ConnectionWindowManager.instance.close(quit: true);
          return true;
        },
      ),
      globalWindowOptions: const WindowOptions(
        size: Size(1, 1),
        hideAppFromTaskbar: true,
        windowButtonVisibility: false,
        titleBarStyle: TitleBarStyle.hidden,
        backgroundColor: Colors.transparent,
      ),
    ),
  );
}

class MainApplication extends StatefulWidget {
  const MainApplication({super.key});

  @override
  MainApplicationState createState() {
    return MainApplicationState();
  }
}

class MainApplicationState extends State<MainApplication> {
  late final ConnectionState connectionState =
      Get.isRegistered<ConnectionState>()
      ? Get.find<ConnectionState>()
      : Get.put(ConnectionState());

  Future<void> handleDeepLink(Uri uri) async {
    if (uri.route != 'connect') return;

    final rawConnection = uri.queryParameters['connection'];
    if (rawConnection == null || rawConnection.isEmpty) return;

    final connection = Connection.fromEncodedString(rawConnection);
    connectionState.addConnection(connection);

    await ConnectionWindowManager.instance.open(focus: true);
  }

  @override
  void initState() {
    super.initState();

    try {
      MultiViewDesktop.allWindowIdsNotifier.addListener(_onWindowIdsChanged);
    } catch (_) {}

    deepLinkService.initialize(handleDeepLink);
  }

  void _onWindowIdsChanged() {
    try {
      final openIds = MultiViewDesktop.allWindowIdsNotifier.value;
      final currentId = ConnectionWindowManager.instance.viewId;
      if (currentId != null && !openIds.contains(currentId)) {
        ConnectionWindowManager.instance.onWindowClosed(currentId);
      }
    } catch (_) {}
  }

  @override
  void dispose() {
    try {
      MultiViewDesktop.allWindowIdsNotifier.removeListener(_onWindowIdsChanged);
    } catch (_) {}
    deepLinkService.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return const SizedBox.shrink();
  }
}
