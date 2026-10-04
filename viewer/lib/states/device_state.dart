import 'package:get/get.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../dio/index.dart' show Api;
import '../dio/models/device.dart' show Device;

enum OrderBy { asc, desc }

enum SortBy { deviceName, timeLastConnected, manual }

class FavoriteState extends GetxController {
  static const _storageKey = 'favorite_device_ids';

  final SharedPreferences preferences;
  final RxList<Device> favorites = <Device>[].obs;
  final RxSet<String> favoriteIds = <String>{}.obs;

  FavoriteState(this.preferences);

  Future<void> load() async {}

  Future<void> loadFavorites() async {
    favoriteIds.assignAll(
      preferences.getStringList(_storageKey)?.toSet() ?? <String>{},
    );
  }

  bool isFavorite(Device device) {
    return favoriteIds.contains(device.id) || device.favorite;
  }

  Future<void> toggle(Device device) async {
    final nextValue = !isFavorite(device);
    device.favorite = nextValue;

    if (nextValue) {
      favoriteIds.add(device.id);
    } else {
      favoriteIds.remove(device.id);
    }

    await preferences.setStringList(_storageKey, favoriteIds.toList());
    favorites.assignAll(Get.find<DeviceState>().devices.where(isFavorite));
  }

  void sync(List<Device> devices) {
    for (final device in devices) {
      device.favorite = isFavorite(device);
    }
    favorites.assignAll(devices.where(isFavorite));
  }

  List<Device> get() => favorites;
}

class DeviceState extends GetxController {
  final RxList<Device> devices = <Device>[].obs;
  final isLoading = false.obs;
  Object? error;

  Future<void> loadDevices() async {
    final Api api = Get.find();
    isLoading.value = true;
    error = null;
    try {
      final List<Device> response = await api.device.getDevices();
      devices.assignAll(response);
      if (Get.isRegistered<FavoriteState>()) {
        Get.find<FavoriteState>().sync(response);
      }
    } catch (value) {
      error = value;
    } finally {
      isLoading.value = false;
    }
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
