import 'package:get/get.dart' show GetxController, Get, Inst;
import 'package:shared_preferences/shared_preferences.dart'
    show SharedPreferences;

import '../../dio/index.dart' show Api;

class ConnectionState extends GetxController {
  late final Api api;
  final Map<String, Future<String>> pending = {};

  ConnectionState() {
    api = Get.find<Api>();
  }

  Future<String> getDeviceToken(String device) {
    final future = pending.putIfAbsent(device, () => loadDeviceToken(device));

    return future.whenComplete(() {
      if (identical(pending[device], future)) {
        pending.remove(device);
      }
    });
  }

  Future<String> loadDeviceToken(String device) async {
    final sharedPreferences = await SharedPreferences.getInstance();

    final key = "token:$device";
    final cache = sharedPreferences.getString(key);
    if (cache != null) return cache;

    final token = await api.organization.createConnectionToken(device);
    await sharedPreferences.setString(key, token);
    return token;
  }
}
