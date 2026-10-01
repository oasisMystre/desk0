import 'package:get/get.dart';
import 'package:flutter/material.dart';
import 'package:flutter_svg/svg.dart';
import 'package:go_router/go_router.dart';

import './sidenav_item.dart';
import '../../states/app_state.dart';

class SideNav extends StatefulWidget {
  final String? selectedRoute;

  const SideNav({super.key, this.selectedRoute});

  @override
  State<SideNav> createState() {
    return SideNavState();
  }
}

class SideNavState extends State<SideNav> {
  late int selectedIndex = 0;

  @override
  void initState() {
    super.initState();
    if (widget.selectedRoute != null) {
      final index = sideNavItems.indexWhere(
        (data) => data.path == widget.selectedRoute,
      );
      if (index != -1) {
        selectedIndex = index;
      }
    }
  }

  final List<SideNavData> sideNavItems = [
    SideNavData(
      icon: (color) {
        return SvgPicture.asset(
          "assets/icons/star.svg",
          colorFilter: ColorFilter.mode(color, BlendMode.srcIn),
        );
      },
      name: "Favorites",
      path: "/",
    ),
    SideNavData(
      icon: (color) {
        return SvgPicture.asset(
          "assets/icons/devices.svg",
          colorFilter: ColorFilter.mode(color, BlendMode.srcIn),
        );
      },
      name: "Devices",
      path: "/devices",
    ),
    SideNavData(
      icon: (color) {
        return SvgPicture.asset(
          "assets/icons/apps.svg",
          colorFilter: ColorFilter.mode(color, BlendMode.srcIn),
        );
      },
      name: "Applications",
      path: "/apps",
    ),
  ];

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();

    final path = GoRouterState.of(context).uri.path;
    final index = sideNavItems.indexWhere((data) => data.path == path);

    if (index != -1) {
      selectedIndex = index;
    }
  }

  @override
  Widget build(BuildContext context) {
    final AppState appState = Get.find();

    return Obx(() {
      const curve = Curves.easeInOut;
      const duration = Duration(milliseconds: 250);

      final bool isOpened = appState.isSideNavOpened.value;
      final double sidebarWidth = MediaQuery.sizeOf(context).width * 0.3;

      return AnimatedContainer(
        duration: duration,
        curve: curve,
        width: isOpened ? sidebarWidth : 0.0,
        child: ClipRect(
          child: OverflowBox(
            minWidth: sidebarWidth,
            maxWidth: sidebarWidth,
            alignment: Alignment.centerLeft,
            child: AnimatedSlide(
              duration: duration,
              curve: curve,
              offset: isOpened ? Offset.zero : const Offset(-1.0, 0.0),
              child: Padding(
                padding: const EdgeInsets.all(6),
                child: Container(
                  decoration: BoxDecoration(
                    color: Colors.white10,
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: Container(
                    padding: const EdgeInsets.all(6),
                    margin: const EdgeInsets.only(top: 32),
                    child: SideNavGroup(
                      selectedIndex: selectedIndex,
                      group: sideNavItems,
                      onSelected: (index, data) {
                        setState(() {
                          selectedIndex = index;
                        });
                        context.go(data.path);
                      },
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
    });
  }
}
