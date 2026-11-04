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
import 'screens/auth.dart';
import 'screens/index.dart';
import 'theme/app_theme.dart';
import 'components/index.dart';
import 'ffi/frb_generated.dart';
import 'l10n/generated/app_localizations.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();
  await LiquidGlassWidgets.initialize();

  final sharedPreferences = await SharedPreferences.getInstance();
  final api = Api(
    baseUrl: Env.apiBaseUrl,
    sharedPreferences: sharedPreferences,
  );

  final AuthState authState = AuthState(
    api: api,
    sharedPreferences: sharedPreferences,
  );

  await authState.init();

  Get.put(api);
  Get.put(authState);
  Get.put(AppState());
  Get.put(DeviceState(api: api));
  Get.put(ApplicationState(api: api));

  runMultiApp(
    home: (context, publicId) => const MainApplication(),
    config: MultiAppConfig(
      generalParams: MultiPlatformParams(closeMode: CloseMode.softCascade),
      globalWindowOptions: WindowOptions(
        size: const Size(720, 480),
        titleBarStyle: TitleBarStyle.hidden,
        windowButtonVisibility: true,
        backgroundColor: Colors.transparent,
        minimumSize: const Size(512, 320),
      ),
    ),
  );
}

final router = GoRouter(
  initialLocation: '/',
  routes: [
    GoRoute(path: '/auth', builder: (context, state) => const AuthScreen()),
    ShellRoute(
      builder: (context, state, child) {
        return LiquidGlassWidgets.wrap(
          child: MainApplication.wrapper(child),
          brightnessResolver: Theme.maybeBrightnessOf,
        );
      },
      routes: [
        GoRoute(path: '/', builder: (context, state) => const HomeScreen()),
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
                SideNav(),
                Expanded(child: child),
              ],
            ),
          ),
          Positioned(top: 0, left: 0, right: 0, child: Toolbar()),
        ],
      ),
    );
  }
}
