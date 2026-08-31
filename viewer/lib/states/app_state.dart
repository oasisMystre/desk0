import 'package:get/get.dart';

enum ListType { grid, list }

class AppState extends GetxController {
  final isSideNavOpened = true.obs;
  final showSearchInput = true.obs;
  final deviceTabListType = ListType.grid.obs;
}
