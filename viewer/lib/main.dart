import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:get/instance_manager.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:multiview_desktop/multiview_desktop.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:liquid_glass_widgets/liquid_glass_widgets.dart';

import 'env.dart';
import 'dio/index.dart';
import 'states/index.dart';
import 'ffi/types/rdp.dart';
import 'screens/index.dart';
import 'theme/app_theme.dart';
import 'components/index.dart';
import 'ffi/frb_generated.dart';
import 'l10n/generated/app_localizations.dart';
import 'components/connection/open_connection.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();
  await LiquidGlassWidgets.initialize();

  final Api api = Api(baseUrl: Env.apiBaseUrl);

  final sharedPreferences = await SharedPreferences.getInstance();
  final guestUser = sharedPreferences.getStringList('user');

  late String user, organization;

  if (guestUser == null) {
    final response = await api.user.createUser();
    await sharedPreferences.setStringList('user', [
      response.user.id,
      response.organization.id,
    ]);
    user = response.user.id;
    organization = response.organization.id;
  } else {
    [user, organization] = guestUser;
  }

  api.updateUserId(user);
  api.updateOrganizationId(organization);

  final deviceState = DeviceState();
  final favoriteState = FavoriteState(sharedPreferences);

  Get.put(api);
  Get.put(AppState());
  Get.put(deviceState);
  Get.put(favoriteState);

  await deviceState.loadDevices();
  await favoriteState.loadFavorites();

  runMultiApp(
    home: (context, publicId) => const MainApplication(),
    config: MultiAppConfig(
      generalParams: MultiPlatformParams(closeMode: CloseMode.softCascade),
      globalWindowOptions: WindowOptions(
        size: const Size(720, 480),
        titleBarStyle: TitleBarStyle.hidden,
        windowButtonVisibility: true,
        backgroundColor: Colors.transparent,
      ),
    ),
  );
}

final router = GoRouter(
  initialLocation: '/',
  routes: [
    ShellRoute(
      builder: (context, state, child) {
        return LiquidGlassWidgets.wrap(
          child: MainApplication.wrapper(child),
          brightnessResolver: Theme.maybeBrightnessOf,
        );
      },
      routes: [
        GoRoute(
          path: '/',
          builder: (context, state) => kDebugMode
              ? const OpenConnection(
                  token: Env.rdpToken,
                  endpoint: Env.rdpEndpoint,
                  connectionType: ConnectionType.terminal,
                  size: SseDesktopSize(width: 1280, height: 720),
                )
              : const HomeScreen(),
        ),
        GoRoute(
          path: '/devices',
          builder: (context, state) => const DevicesScreen(),
        ),
        GoRoute(path: '/apps', builder: (context, state) => const AppsScreen()),
      ],
    ),
  ],
);

class MainApplication extends StatelessWidget {
  const MainApplication({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: 'Desk0',
      supportedLocales: AppLocalizations.supportedLocales,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      debugShowCheckedModeBanner: false,
      routerConfig: router,
      theme: ThemeData(
        brightness: Brightness.dark,
        textTheme: GoogleFonts.monaSansTextTheme(),
        scaffoldBackgroundColor: Colors.transparent,
        colorScheme: const ColorScheme.dark(
          primary: AppColors.accentBlue,
          surface: AppColors.mainBackground,
        ),
        iconButtonTheme: IconButtonThemeData(
          style: ButtonStyle(
            minimumSize: WidgetStatePropertyAll(Size(32, 32)),
            iconSize: WidgetStatePropertyAll(18.0),
            padding: WidgetStatePropertyAll(EdgeInsets.all(0)),
          ),
        ),
        useMaterial3: true,
      ),
    );
  }

  static Widget wrapper(Widget child) {
    return GlassScaffold(
      backgroundColor: Colors.transparent,
      settings: LiquidGlassSettings.figma(
        frost: 5,
        depth: 10,
        dispersion: 5,
        refraction: 10,
      ),
      body: Stack(
        children: [
          Positioned.fill(
            child: Row(
              children: [
                if (!kDebugMode) SideNav(),
                Expanded(child: child),
              ],
            ),
          ),
          if (!kDebugMode)
            Positioned(top: 0, left: 0, right: 0, child: Toolbar()),
        ],
      ),
    );
  }
}
