import 'package:flutter/widgets.dart';
import 'package:get/get.dart' show Get, Inst, Obx;
import 'package:flutter_svg/svg.dart' show SvgPicture;

import '../../states/app_state.dart' show AppState, ListType;
import '../../components/toolbar/tab_icon_button.dart'
    show TabIconGroupButton, TabIconData;

class TabListType extends StatelessWidget {
  const TabListType({super.key});

  @override
  Widget build(BuildContext context) {
    final AppState appState = Get.find();

    return Obx(() {
      return TabIconGroupButton(
        selectedIndex: appState.deviceTabListType.value == ListType.grid
            ? 0
            : 1,
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
        onSelected: (index, data) {
          switch (index) {
            case 0:
              appState.deviceTabListType.value = ListType.grid;
              break;
            case 1:
              appState.deviceTabListType.value = ListType.list;
          }
        },
      );
    });
  }
}
