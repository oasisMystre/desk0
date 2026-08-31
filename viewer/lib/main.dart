import 'package:flutter/material.dart';
import 'package:get/instance_manager.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:liquid_glass_widgets/liquid_glass_widgets.dart';
import 'package:window_manager/window_manager.dart';

import 'screens/index.dart';
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
  await windowManager.ensureInitialized();
  await LiquidGlassWidgets.initialize();

  windowManager.addListener(InterceptFullscreenListener());

  Get.put(AppState());

  await windowManager.waitUntilReadyToShow(
    WindowOptions(
      size: Size(640, 420),
      minimumSize: Size(640, 420),
      maximumSize: Size(640, 420),
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
          body: Stack(
            children: [
              Positioned.fill(
                child: Row(
                  children: [
                    SideNav(),
                    Expanded(
                      child: Padding(
                        padding: EdgeInsetsGeometry.all(4),
                        child: Column(children: [Expanded(child: child)]),
                      ),
                    ),
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
        textTheme: GoogleFonts.robotoFlexTextTheme(),
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
