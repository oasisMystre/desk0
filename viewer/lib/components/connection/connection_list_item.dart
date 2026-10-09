import 'package:flutter/material.dart';
import 'package:flutter_svg/svg.dart' show SvgPicture;

import '../../dio/index.dart' show Device;
import 'connection_menu.dart' show ConnectionMenu;
import 'platform_image_card.dart' show PlatformImageCard;

class ConnectionListItem extends StatelessWidget {
  final Device device;

  const ConnectionListItem({super.key, required this.device});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: Container(
        width: 280,
        decoration: BoxDecoration(
          color: theme.colorScheme.onSurfaceVariant.withValues(alpha: 0.05),
          borderRadius: BorderRadius.all(Radius.circular(8)),
        ),
        child: Row(
          spacing: 8,
          children: [
            SizedBox(
              width: 46,
              height: double.infinity,
              child: PlatformImageCard(
                os: device.os,
                borderRadius: BorderRadius.only(
                  topLeft: Radius.circular(8),
                  bottomLeft: Radius.circular(8),
                ),
              ),
            ),
            Expanded(
              child: Row(
                children: [
                  Expanded(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            const Badge(),
                            const SizedBox(width: 4),
                            if (device.username != null)
                              Text(
                                device.username!,
                                style: theme.textTheme.bodySmall?.copyWith(
                                  color: Colors.white,
                                  fontWeight: FontWeight.w600,
                                  height: 1.2,
                                ),
                                overflow: TextOverflow.ellipsis,
                              ),
                          ],
                        ),
                        if (device.hostname != null)
                          Text(
                            device.hostname!,
                            style: theme.textTheme.labelSmall?.copyWith(
                              color: Colors.white70,
                              fontWeight: FontWeight.w300,
                              height: 1.0,
                            ),
                            overflow: TextOverflow.ellipsis,
                          ),
                      ],
                    ),
                  ),
                  ConnectionMenu(
                    device: device,
                    trigger: (toggle) => IconButton(
                      icon: SvgPicture.asset(
                        "assets/icons/more_vert.svg",
                        width: 16,
                        height: 16,
                        colorFilter: const ColorFilter.mode(
                          Colors.white,
                          BlendMode.srcIn,
                        ),
                      ),
                      onPressed: toggle,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
