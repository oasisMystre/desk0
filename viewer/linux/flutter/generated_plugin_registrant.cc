//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <desktop_multi_window/desktop_multi_window_plugin.h>
#include <flutter_custom_cursor/flutter_custom_cursor_plugin.h>
#include <screen_retriever_linux/screen_retriever_linux_plugin.h>
#include <texture_rgba_renderer/texture_rgba_renderer_plugin.h>
#include <window_manager/window_manager_plugin.h>
#include <window_size/window_size_plugin.h>

void fl_register_plugins(FlPluginRegistry* registry) {
  g_autoptr(FlPluginRegistrar) desktop_multi_window_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "DesktopMultiWindowPlugin");
  desktop_multi_window_plugin_register_with_registrar(desktop_multi_window_registrar);
  g_autoptr(FlPluginRegistrar) flutter_custom_cursor_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "FlutterCustomCursorPlugin");
  flutter_custom_cursor_plugin_register_with_registrar(flutter_custom_cursor_registrar);
  g_autoptr(FlPluginRegistrar) screen_retriever_linux_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "ScreenRetrieverLinuxPlugin");
  screen_retriever_linux_plugin_register_with_registrar(screen_retriever_linux_registrar);
  g_autoptr(FlPluginRegistrar) texture_rgba_renderer_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "TextureRgbaRendererPlugin");
  texture_rgba_renderer_plugin_register_with_registrar(texture_rgba_renderer_registrar);
  g_autoptr(FlPluginRegistrar) window_manager_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "WindowManagerPlugin");
  window_manager_plugin_register_with_registrar(window_manager_registrar);
  g_autoptr(FlPluginRegistrar) window_size_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "WindowSizePlugin");
  window_size_plugin_register_with_registrar(window_size_registrar);
}
