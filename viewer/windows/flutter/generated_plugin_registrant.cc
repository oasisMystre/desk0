//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <flutter_custom_cursor/flutter_custom_cursor_plugin.h>
#include <multiview_desktop/multi_view_desktop_plugin.h>
#include <texture_rgba_renderer/texture_rgba_renderer_plugin_c_api.h>

void RegisterPlugins(flutter::PluginRegistry* registry) {
  FlutterCustomCursorPluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("FlutterCustomCursorPlugin"));
  MultiViewDesktopPluginRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("MultiViewDesktopPlugin"));
  TextureRgbaRendererPluginCApiRegisterWithRegistrar(
      registry->GetRegistrarForPlugin("TextureRgbaRendererPluginCApi"));
}
