import 'package:flutter/material.dart';
import 'package:get/instance_manager.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:liquid_glass_widgets/liquid_glass_widgets.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:window_manager/window_manager.dart';

import 'screens/index.dart';
import 'dio/index.dart';
import 'theme/app_theme.dart';
import 'components/index.dart';
import 'states/app_state.dart';

class InterceptFullscreenListener extends WindowListener {
  @override
  void onWindowEnterFullScreen() async {
    await windowManager.setFullScreen(false);
  }
}

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final Api api = Api(baseUrl: 'http://0.0.0.0:8000');
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

  await windowManager.ensureInitialized();
  await LiquidGlassWidgets.initialize();

  windowManager.addListener(InterceptFullscreenListener());

  Get.put(api);
  Get.put(AppState());

  await windowManager.waitUntilReadyToShow(
    WindowOptions(
      size: Size(720, 480),
      skipTaskbar: true,
      backgroundColor: Colors.transparent,
      titleBarStyle: TitleBarStyle.hidden,
    ),
    () async {
      await windowManager.show();
      await windowManager.focus();
    },
  );

  runApp(LiquidGlassWidgets.wrap(child: const MainApplication()));
}

final router = GoRouter(
  initialLocation: '/',
  routes: [
    ShellRoute(
      builder: (context, state, child) {
        return GlassScaffold(
          backgroundColor: Colors.transparent,
          settings: LiquidGlassSettings.figma(
            depth: 10,
            dispersion: 5,
            frost: 5,
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
          themeOverride: GlassThemeData(
            dark: GlassThemeVariant(settings: GlassThemeSettings()),
          ),
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
}
