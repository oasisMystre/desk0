import 'package:desk0/states/app_state.dart';
import 'package:flutter/material.dart';
import 'package:flutter_svg/svg.dart';
import 'package:get/get.dart';

import '../search_input.dart';
import './tab_icon_button.dart';

class Toolbar extends StatelessWidget {
  const Toolbar({super.key});

  @override
  Widget build(BuildContext context) {
    final AppState appState = Get.find();

    return Row(
      children: [
        Container(
          margin: EdgeInsetsGeometry.directional(top: 0, start: 96),
          child: IconButton(
            icon: SvgPicture.asset(
              'assets/icons/sidebar_toggle.svg',
              colorFilter: ColorFilter.mode(Colors.white30, BlendMode.srcIn),
            ),
            onPressed: () {
              appState.isSideNavOpened.value = !appState.isSideNavOpened.value;
            },
          ),
        ),
        Expanded(
          child: Padding(
            padding: EdgeInsetsGeometry.directional(top: 4),
            child: Row(
              spacing: 8,
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TabIconButton(
                  data: TabIconData(
                    name: "Add connection",

                    icon: (color) {
                      return SvgPicture.asset(
                        'assets/icons/add.svg',
                        colorFilter: color == null
                            ? null
                            : ColorFilter.mode(color, BlendMode.srcIn),
                      );
                    },
                  ),
                  onPressed: () {},
                ),
                TabIconButton(
                  data: TabIconData(
                    name: "Notifications",
                    icon: (color) {
                      return SvgPicture.asset(
                        'assets/icons/bell.svg',
                        colorFilter: color == null
                            ? null
                            : ColorFilter.mode(color, BlendMode.srcIn),
                      );
                    },
                  ),
                  onPressed: () {},
                ),
                TabIconGroupButton(
                  selectedIndex: 0,
                  group: [
                    TabIconData(
                      name: "Grid",
                      icon: (color) {
                        return SvgPicture.asset(
                          'assets/icons/grid.svg',
                          colorFilter: color == null
                              ? null
                              : ColorFilter.mode(color, BlendMode.srcIn),
                        );
                      },
                    ),
                    TabIconData(
                      icon: (color) {
                        return SvgPicture.asset(
                          'assets/icons/list.svg',
                          colorFilter: color == null
                              ? null
                              : ColorFilter.mode(color, BlendMode.srcIn),
                        );
                      },
                      name: "List",
                    ),
                  ],
                  onSelected: (index, data) {},
                ),
                TabIconGroupButton(
                  selectedIndex: 0,
                  group: [
                    TabIconData(
                      icon: (color) {
                        return SvgPicture.asset(
                          'assets/icons/sort.svg',
                          colorFilter: color == null
                              ? null
                              : ColorFilter.mode(color, BlendMode.srcIn),
                        );
                      },
                      name: "Sort",
                    ),
                    TabIconData(
                      icon: (color) {
                        return SvgPicture.asset(
                          'assets/icons/filter.svg',
                          colorFilter: color == null
                              ? null
                              : ColorFilter.mode(color, BlendMode.srcIn),
                        );
                      },
                      name: "Filter",
                    ),
                  ],
                  onSelected: (index, data) {},
                ),
                Flexible(child: SearchInput()),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
