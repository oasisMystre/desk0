import 'package:flutter/material.dart';

class AppColors {
  // Window & Surfaces
  static const Color windowBackground = Color(0xFF14161A);
  static const Color sidebarBackground = Color(0xFF1A1C22);
  static const Color sidebarBorder = Color(0xFF2A2D35);
  static const Color mainBackground = Color(0xFF131518);

  // Glass Elements
  static const Color glassSurface = Color(0x1AFFFFFF); // ~10% white
  static const Color glassSurfaceHover = Color(0x28FFFFFF); // ~16% white
  static const Color glassSurfaceActive = Color(0x38FFFFFF); // ~22% white
  static const Color glassBorder = Color(0x1FFFFFFF); // ~12% white border
  static const Color glassBorderHover = Color(0x3DFFFFFF); // ~24% white border
  static const Color glassOverlayBackground = Color(
    0xF216181F,
  ); // ~95% dark frosted glass for menus/dialogs

  // Sidebar Selected & Hover State (Translucent glass pill)
  static const Color sidebarSelected = Color(
    0x2BFFFFFF,
  ); // Frosted highlight pill
  static const Color sidebarSelectedBorder = Color(0x26FFFFFF);
  static const Color sidebarHover = Color(0x14FFFFFF);

  // Accent Colors
  static const Color accentBlue = Color(0xFF0A84FF);

  // Toolbar
  static const Color toolbarButtonBg = Color(0x14FFFFFF);
  static const Color toolbarButtonBorder = Color(0x1FFFFFFF);
  static const Color toolbarButtonHover = Color(0x26FFFFFF);
  static const Color toolbarActiveSegment = Color(0x33FFFFFF);

  // Search
  static const Color searchBackground = Color(0x1FFFFFFF);
  static const Color searchBorder = Color(0x26FFFFFF);
  static const Color searchBorderFocused = Color(0x4DFFFFFF);

  // Text
  static const Color textPrimary = Color(0xFFFFFFFF);
  static const Color textSecondary = Color(0xFF9EABB8);
  static const Color textMuted = Color(0xFF6B7280);
}
