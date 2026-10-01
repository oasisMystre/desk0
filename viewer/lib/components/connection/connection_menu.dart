import 'package:flutter/material.dart';
import 'package:flutter_svg/svg.dart' show SvgPicture;
import 'package:liquid_glass_widgets/liquid_glass_widgets.dart'
    show
        GlassMenu,
        GlassButton,
        GlassMenuItem,
        GlassContainer,
        LiquidRoundedRectangle;

import '../../theme/app_theme.dart' show AppColors;

class ConnectionMenu extends StatelessWidget {
  final Widget Function(void Function() toggle)? trigger;

  const ConnectionMenu({super.key, this.trigger});

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
          onTap: () {},
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
          title: "Add to Favorites",
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
          onTap: () {},
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
                return DeleteConfirmationDialog();
              },
            );
          },
        ),
      ],
    );
  }
}

class DeleteConfirmationDialog extends StatelessWidget {
  const DeleteConfirmationDialog({super.key});

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
                    onPressed: () {},
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
