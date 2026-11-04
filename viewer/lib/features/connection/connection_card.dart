import 'package:flutter/material.dart';
import 'package:get/get.dart' show Get, Inst;
import 'package:flutter_svg/flutter_svg.dart' show SvgPicture;
import 'package:liquid_glass_widgets/widgets/interactive/glass_button.dart'
    show GlassButton;

import 'connection_menu.dart' show ConnectionMenu;
import '../../dio/models/device.dart' show Device;
import '../../states/device_state.dart' show DeviceState;
import 'platform_image_card.dart' show PlatformImageCard;
import '../../error.dart' show handleHttpErrorFromContext;

class ConnectionCard extends StatefulWidget {
  final Device device;

  const ConnectionCard({super.key, required this.device});

  @override
  State<StatefulWidget> createState() {
    return ConnectionCardState();
  }
}

class ConnectionCardState extends State<ConnectionCard> {
  bool hovered = false;

  late final Device device = widget.device;
  final DeviceState deviceState = Get.find<DeviceState>();

  void toggleHovered(bool? value) {
    if (value != null) {
      setState(() {
        hovered = value;
      });
      return;
    }

    setState(() {
      hovered = !hovered;
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return MouseRegion(
      onEnter: (event) {
        toggleHovered(true);
      },
      onExit: (event) {
        toggleHovered(false);
      },
      child: Stack(
        children: [
          Positioned.fill(
            child: PlatformImageCard(
              os: device.os,
              borderRadius: BorderRadius.all(Radius.circular(8)),
            ),
          ),
          Positioned(
            top: 8,
            left: 8,
            child: Badge(backgroundColor: Colors.green),
          ),
          Positioned(
            right: 4,
            top: 4,
            child: AnimatedOpacity(
              duration: const Duration(milliseconds: 150),
              opacity: hovered ? 1 : 0,
              child: Row(
                spacing: 8,
                children: [
                  ConnectionMenu(device: device),
                  GlassButton(
                    width: 24,
                    height: 24,
                    iconSize: 16,
                    icon: SvgPicture.asset(
                      "assets/icons/star.svg",
                      width: 14,
                      height: 14,
                      colorFilter: const ColorFilter.mode(
                        Colors.white70,
                        BlendMode.srcIn,
                      ),
                    ),
                    onTap: () async {
                      await handleHttpErrorFromContext(
                        context,
                        () => deviceState.toggleFavorite(device.id),
                        "Failed to add device to favorites",
                      );

                      if (mounted) setState(() {});
                    },
                  ),
                ],
              ),
            ),
          ),
          Positioned(
            bottom: 4,
            left: 4,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (device.username != null)
                  Text(
                    device.username!,
                    style: theme.textTheme.bodySmall?.copyWith(
                      height: 1.2,
                      color: Colors.white,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                if (device.os != null)
                  Text(
                    device.os!,
                    style: theme.textTheme.labelSmall?.copyWith(
                      height: 1.0,
                      fontSize: 9,
                      color: Colors.white70,
                      fontWeight: FontWeight.w300,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
