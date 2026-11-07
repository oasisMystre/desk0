//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <app_links/app_links_plugin_c_api.h>
#include <flutter_custom_cursor/flutter_custom_cursor_plugin.h>
#include <irondash_engine_context/irondash_engine_context_plugin_c_api.h>
#include <multiview_desktop/multi_view_desktop_plugin.h>
#include <system_tray/system_tray_plugin.h>
#include <url_launcher_windows/url_launcher_windows.h>

void RegisterPlugins(flutter::PluginRegistry* registry) {
  AppLinksPluginCApiRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("AppLinksPluginCApi"));
  FlutterCustomCursorPluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("FlutterCustomCursorPlugin"));
  IrondashEngineContextPluginCApiRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("IrondashEngineContextPluginCApi"));
  MultiViewDesktopPluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("MultiViewDesktopPlugin"));
  SystemTrayPluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("SystemTrayPlugin"));
  UrlLauncherWindowsRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("UrlLauncherWindows"));
}
