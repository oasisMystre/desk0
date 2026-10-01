import 'package:get/get.dart';

import '../dio/index.dart' show Api;
import '../dio/models/device.dart' show Device;

enum OrderBy { asc, desc }

enum SortBy { deviceName, timeLastConnected, manual }

class FavoriteState extends GetxController {
  final RxList<Device> favorites = <Device>[].obs;

  Future<void> load() async {}

  void addDevice(Device device) {
    favorites.add(device);
  }

  void removeDevice(String device) {
    favorites.removeWhere((value) => value.id == device);
  }

  List<Device> get() => favorites;
}

class DeviceState extends GetxController {
  final RxList<Device> devices = <Device>[].obs;

  Future<void> load() async {
    final Api api = Get.find();
    final List<Device> response = await api.device.getDevices();
    devices.clear();
    devices.addAll(response);
  }

  Future<void> deleteDevice(String device) async {
    final Api api = Get.find();
    final Device response = await api.device.deleteDevice(device);
    devices.removeWhere((value) => value.id == response.id);
  }

  void upsertDevice(Device device) {
    final int index = devices.indexWhere((value) => value.id == device.id);
    if (index == -1) {
      devices.add(device);
    } else {
      devices[index] = device;
    }
  }

  List<Device> get() => devices;
}
