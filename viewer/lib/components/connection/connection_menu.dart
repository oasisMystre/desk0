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
import '../../theme/app_theme.dart' show AppColors;
import '../../states/index.dart' show DeviceState, FavoriteState;
import 'open_connection.dart' show OpenConnection, ConnectionType;

class ConnectionMenu extends StatelessWidget {
  final Device device;
  final Widget Function(void Function() toggle)? trigger;

  const ConnectionMenu({super.key, required this.device, this.trigger});

  Future<void> openViewer(BuildContext context) async {
    final api = Get.find<Api>();
    final desktopSize = device.metadata.size;

    try {
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
          title: device.hostname ?? device.username ?? 'Desk0 Viewer',
          size: Size(
            desktopSize.width.toDouble(),
            desktopSize.height.toDouble(),
          ),
          minimumSize: const Size(640, 360),
          titleBarStyle: TitleBarStyle.normal,
          windowButtonVisibility: true,
        ),
      );
    } catch (error) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('Unable to connect: $error')));
    }
  }

  Future<void> _toggleFavorite() async {
    await Get.find<FavoriteState>().toggle(device);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

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
          title: "Connect",
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
          title: "Terminal",
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
          title: "Install App",
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
          title: device.favorite ? "Remove from Favorites" : "Add to Favorites",
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
          onTap: _toggleFavorite,
        ),
        GlassMenuItem(
          title: "Delete",
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

  const DeleteConfirmationDialog({super.key, required this.device});

  @override
  Widget build(BuildContext context) {
    final theme = TextTheme.of(context);

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
                    "Delete device?",
                    style: theme.titleSmall?.copyWith(
                      color: AppColors.textPrimary,
                      fontWeight: FontWeight.w600,
                    ),
                  ),

                  Text(
                    "Device will be deleted permanently.",
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
                    child: Text("Cancel"),
                  ),
                ),
                Expanded(
                  child: TextButton(
                    style: TextButton.styleFrom(
                      backgroundColor: Colors.redAccent,
                    ),
                    child: Text(
                      "Delete",
                      style: TextStyle(color: AppColors.textPrimary),
                    ),
                    onPressed: () async {
                      try {
                        await Get.find<DeviceState>().deleteDevice(device.id);
                        if (context.mounted) Navigator.of(context).pop();
                      } catch (error) {
                        if (!context.mounted) return;
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            content: Text('Unable to delete device: $error'),
                          ),
                        );
                      }
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
