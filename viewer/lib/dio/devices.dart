import 'package:dio_cache_interceptor/dio_cache_interceptor.dart'
    show CacheOptionExtension;

import 'api_impl.dart' show ApiImpl;
import 'models/device.dart' show Device;
import 'models/application.dart' show Application;
import 'cache_options.dart' show cacheOptions;

class DeviceApi extends ApiImpl {
  const DeviceApi({required super.client});

  Future<List<Device>> getDevices() async {
    final response = await client.get(
      '/devices',
      options: (await cacheOptions).toOptions(),
    );
    final result = response.data as List<dynamic>;
    return result
        .map((data) => Device.fromJson(data as Map<String, dynamic>))
        .toList();
  }

  Future<Device> updateDevice(String device, Map<String, dynamic> data) async {
    final response = await client.post('/devices/$device', data: data);

    final result = response.data as Map<String, dynamic>;
    return Device.fromJson(result);
  }

  Future<Device> deleteDevice(String id) async {
    final response = await client.delete('/devices/$id');

    final result = response.data as Map<String, dynamic>;
    return Device.fromJson(result);
  }

  Future<List<Application>> getApplications(String deviceId) async {
    final response = await client.get(
      '/devices/$deviceId/applications',
      options: (await cacheOptions).toOptions(),
    );
    final result = response.data as List<dynamic>;
    return result
        .map((data) => Application.fromJson(data as Map<String, dynamic>))
        .toList();
  }

  Future<void> addApplication(String deviceId, String applicationId) async {
    await client.post(
      '/devices/$deviceId/applications',
      data: {'application': applicationId},
    );
  }

  Future<void> removeApplication(String deviceId, String applicationId) async {
    await client.delete('/devices/$deviceId/applications/$applicationId');
  }
}
