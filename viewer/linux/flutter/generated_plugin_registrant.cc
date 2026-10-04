//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <flutter_custom_cursor/flutter_custom_cursor_plugin.h>
#include <multiview_desktop/multiview_desktop_plugin.h>
#include <texture_rgba_renderer/texture_rgba_renderer_plugin.h>

void fl_register_plugins(FlPluginRegistry* registry) {
  g_autoptr(FlPluginRegistrar) flutter_custom_cursor_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "FlutterCustomCursorPlugin");
  flutter_custom_cursor_plugin_register_with_registrar(flutter_custom_cursor_registrar);
  g_autoptr(FlPluginRegistrar) multiview_desktop_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "MultiviewDesktopPlugin");
  multiview_desktop_plugin_register_with_registrar(multiview_desktop_registrar);
  g_autoptr(FlPluginRegistrar) texture_rgba_renderer_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "TextureRgbaRendererPlugin");
  texture_rgba_renderer_plugin_register_with_registrar(texture_rgba_renderer_registrar);
}
