import 'package:get/get.dart';
import 'package:multiview_desktop/multiview_desktop.dart';
import 'package:flutter/material.dart' hide ConnectionState;

import 'theme/app_theme.dart' show AppColors;
import 'ffi/frb_generated.dart' show RustLib;
import 'features/connection/index.dart' show OpenConnection;
import 'states/index.dart' show ConnectionState, Connection;
import 'deep_link_service.dart' show DeepLinkService, DeepLinkUri;
import 'l10n/generated/app_localizations.dart' show AppLocalizations;

final DeepLinkService deepLinkService = DeepLinkService();

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();

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
            primary: AppColors.accentBlue,
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
      generalParams: const MultiPlatformParams(
        enableDynamicAnchor: true,
        closeMode: CloseMode.softCascade,
      ),
      macosParams: const MacosPlatformParams(
        closeAppAfterLastWindowClosed: false,
        saveLastWindowToReopen: false,
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
  Future<int>? window;

  final ConnectionState connectionState = ConnectionState();

  Future<void> handleDeepLink(Uri uri) async {
    if (uri.route != 'connect') return;

    final rawConnection = uri.queryParameters['connection'];
    if (rawConnection == null || rawConnection.isEmpty) return;

    final connection = Connection.fromEncodedString(rawConnection);
    connectionState.addConnection(connection);

    if (window == null) {
      window = openWindow(
        (context, publicId) => const OpenConnection(),
        options: const WindowOptions(
          title: "Desk0 Viewer",
          windowButtonVisibility: true,
          minimumSize: Size(640, 400),
          titleBarStyle: TitleBarStyle.hidden,
          backgroundColor: Colors.transparent,
        ),
      );
    }
  }

  @override
  void initState() {
    super.initState();

    Get.put(connectionState);
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      try {
        await MultiViewDesktop.of(context).hide();
      } catch (_) {}
    });

    deepLinkService.initialize(handleDeepLink);
  }

  @override
  void dispose() {
    deepLinkService.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return const SizedBox.shrink();
  }
}
