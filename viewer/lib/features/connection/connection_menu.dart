import 'package:flutter/material.dart';
import 'package:get/get.dart' show Get, Inst;
import 'package:flutter_svg/svg.dart' show SvgPicture;
import 'package:multiview_desktop/multiview_desktop.dart'
    show openWindow, WindowOptions, TitleBarStyle;

import 'package:liquid_glass_widgets/liquid_glass_widgets.dart'
    show
        GlassMenu,
        GlassButton,
        GlassMenuItem,
        GlassContainer,
        LiquidRoundedRectangle;

import '../../env.dart' show Env;
import '../../dio/index.dart' show Api, Device;
import '../../states/index.dart' show DeviceState;
import '../../theme/app_theme.dart' show AppColors;
import '../../error.dart' show handleHttpErrorFromContext;
import 'open_connection.dart' show OpenConnection, ConnectionType;
import '../../l10n/generated/app_localizations.dart' show AppLocalizations;

class ConnectionMenu extends StatelessWidget {
  final Device device;
  final Widget Function(void Function() toggle)? trigger;

  late final DeviceState deviceState = Get.find();

  ConnectionMenu({super.key, required this.device, this.trigger});

  Future<void> openViewer(BuildContext context) async {
    final api = Get.find<Api>();
    final desktopSize = device.metadata.size;

    handleHttpErrorFromContext(context, () async {
      final token = await api.organization.createConnectionToken(device.id);
      if (!context.mounted) return;

      await openWindow(
        (_, _) => OpenConnection(
          token: token,
          size: desktopSize,
          endpoint: Env.rdpEndpoint,
          connectionType: ConnectionType.desktop,
        ),
        parentContext: context,
        options: WindowOptions(
          title: device.hostname != null ? device.hostname : device.username,
          windowButtonVisibility: true,
          minimumSize: const Size(640, 360),
          titleBarStyle: TitleBarStyle.normal,
          size: Size(
            desktopSize.width.toDouble(),
            desktopSize.height.toDouble(),
          ),
        ),
      );
    }, "Unable to connect to device");
  }

  Future<void> toggleFavorite(BuildContext context) async {
    await handleHttpErrorFromContext(
      context,
      () => deviceState.toggleFavorite(device.id),
      "Failed to add device to favorites",
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final i10n = AppLocalizations.of(context)!;

    return GlassMenu(
      itemBorderRadius: 0,
      menuBorderRadius: 24,
      triggerBuilder: (context, toggle) {
        return trigger == null
            ? GlassButton(
                width: 24,
                height: 24,
                iconSize: 16,
                icon: SvgPicture.asset(
                  "assets/icons/more_vert.svg",
                  width: 16,
                  height: 16,
                  colorFilter: const ColorFilter.mode(
                    Colors.white,
                    BlendMode.srcIn,
                  ),
                ),
                onTap: () {
                  toggle();
                },
              )
            : trigger!(toggle);
      },
      items: [
        GlassMenuItem(
          title: i10n.connect_button,
          icon: SvgPicture.asset(
            "assets/icons/connect.svg",
            width: 20,
            height: 20,
            colorFilter: const ColorFilter.mode(
              Colors.white70,
              BlendMode.srcIn,
            ),
          ),
          titleStyle: theme.textTheme.labelMedium?.copyWith(
            color: Colors.white70,
          ),
          onTap: () => openViewer(context),
        ),
        GlassMenuItem(
          title: i10n.terminal_button,
          icon: SvgPicture.asset(
            "assets/icons/terminal.svg",
            width: 20,
            height: 20,
            colorFilter: const ColorFilter.mode(
              Colors.white70,
              BlendMode.srcIn,
            ),
          ),
          titleStyle: theme.textTheme.labelMedium?.copyWith(
            color: Colors.white70,
          ),
          onTap: () {},
        ),
        GlassMenuItem(
          title: i10n.install_app_button,
          icon: SvgPicture.asset(
            "assets/icons/install_app.svg",
            width: 20,
            height: 20,
            colorFilter: const ColorFilter.mode(
              Colors.white70,
              BlendMode.srcIn,
            ),
          ),
          titleStyle: theme.textTheme.labelMedium?.copyWith(
            color: Colors.white70,
          ),
          onTap: () {},
        ),
        GlassMenuItem(
          title: device.favorite
              ? i10n.remove_favorite_title
              : i10n.add_favorite_title,
          icon: SvgPicture.asset(
            "assets/icons/star.svg",
            width: 20,
            height: 20,
            colorFilter: const ColorFilter.mode(
              Colors.white70,
              BlendMode.srcIn,
            ),
          ),
          titleStyle: theme.textTheme.labelMedium?.copyWith(
            color: Colors.white70,
          ),
          onTap: () => toggleFavorite(context),
        ),
        GlassMenuItem(
          title: i10n.delete_button,
          icon: SvgPicture.asset(
            "assets/icons/trash.svg",
            width: 20,
            height: 20,
            colorFilter: const ColorFilter.mode(Colors.red, BlendMode.srcIn),
          ),
          titleStyle: theme.textTheme.labelMedium?.copyWith(color: Colors.red),
          onTap: () {
            showDialog(
              context: context,
              builder: (context) {
                return DeleteConfirmationDialog(device: device);
              },
            );
          },
        ),
      ],
    );
  }
}

class DeleteConfirmationDialog extends StatelessWidget {
  final Device device;
  late final DeviceState deviceState = Get.find();

  DeleteConfirmationDialog({super.key, required this.device});

  @override
  Widget build(BuildContext context) {
    final theme = TextTheme.of(context);
    final i10n = AppLocalizations.of(context)!;

    return Dialog(
      elevation: 0,
      backgroundColor: Colors.transparent,
      child: GlassContainer(
        width: 250,
        height: 124,
        padding: EdgeInsetsGeometry.all(16),
        shape: LiquidRoundedRectangle(borderRadius: 24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Column(
                spacing: 2,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    i10n.delete_dialog_title,
                    style: theme.titleSmall?.copyWith(
                      color: AppColors.textPrimary,
                      fontWeight: FontWeight.w600,
                    ),
                  ),

                  Text(
                    i10n.delete_dialog_subtitle,
                    style: theme.bodySmall?.copyWith(
                      color: AppColors.textSecondary,
                      height: 1.3,
                    ),
                  ),
                ],
              ),
            ),
            Row(
              spacing: 16,
              children: [
                Expanded(
                  child: TextButton(
                    style: TextButton.styleFrom(
                      backgroundColor: AppColors.glassSurface,
                      foregroundColor: AppColors.textPrimary,
                    ),
                    onPressed: () {
                      Navigator.of(context).pop();
                    },
                    child: Text(i10n.cancel_button),
                  ),
                ),
                Expanded(
                  child: TextButton(
                    style: TextButton.styleFrom(
                      backgroundColor: Colors.redAccent,
                    ),
                    child: Text(
                      i10n.delete_button,
                      style: TextStyle(color: AppColors.textPrimary),
                    ),
                    onPressed: () async {
                      await handleHttpErrorFromContext(
                        context,
                        () => deviceState.deleteDevice(device.id),
                        "Failed to delete device",
                      );
                    },
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
