import 'dart:developer';

import 'package:dio/dio.dart';
import 'package:get/get.dart'
    show Get, Inst, RxList, GetxController, ListExtension, BoolExtension;

import '../dio/index.dart' show Api;
import '../dio/models/device.dart' show Device;

enum OrderBy { asc, desc }

enum SortBy { deviceName, timeLastConnected, manual }

class DeviceState extends GetxController {
  final Api api;
  final RxList<Device> devices = <Device>[].obs;
  final RxList<Device> favorites = <Device>[].obs;

  Object? error;
  final isLoading = false.obs;

  DeviceState({required this.api});

  Future<void> loadDevices() async {
    isLoading.value = true;
    error = null;
    try {
      final List<Device> response = await api.device.getDevices();
      devices.assignAll(response);
    } on DioException catch (response, stackTrace) {
      final data = response.response?.data;
      final message = data is Map ? data['message'] as String? : null;

      error = message == null ? "Failed to load devices" : message;

      log(
        'Failed to load devices',
        name: 'desk0.devices',
        error: response,
        stackTrace: stackTrace,
      );
    } catch (value, stackTrace) {
      error = 'Unexpected error loading devices';

      log(
        'Failed to load devices',
        name: 'desk0.devices',
        error: value,
        stackTrace: stackTrace,
      );
    } finally {
      isLoading.value = false;
    }
  }

  Future<void> toggleFavorite(String deviceId) async {
    final int index = devices.indexWhere((value) => value.id == deviceId);
    if (index == -1) return;

    final device = devices[index];
    final previousFavoriteValue = device.favorite;

    device.favorite = !previousFavoriteValue;
    devices.refresh();

    try {
      final Api api = Get.find();
      final updated = await api.device.updateDevice(device.id, {
        'favorite': device.favorite,
      });

      devices[index] = updated;
    } catch (error, stackTrace) {
      device.favorite = previousFavoriteValue;
      devices.refresh();

      log(
        'Failed to toggle device favorite',
        name: 'desk0.devices',
        error: error,
        stackTrace: stackTrace,
      );

      rethrow;
    }
  }

  Future<void> deleteDevice(String device) async {
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
