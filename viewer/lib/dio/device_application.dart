import 'package:dio_cache_interceptor/dio_cache_interceptor.dart'
    show CacheOptionExtension;

import 'api_impl.dart' show ApiImpl;
import 'cache_options.dart' show cacheOptions;
import 'models/index.dart' show DeviceApplication, ExpandedDeviceApplication;

class DeviceApplicationApi extends ApiImpl {
  DeviceApplicationApi({required super.client});

  Future<List<ExpandedDeviceApplication>> getDeviceApplications(
    String deviceId,
  ) async {
    final response = await client.get(
      '/devices/$deviceId/applications',
      options: (await cacheOptions).toOptions(),
    );
    final result = response.data as List<dynamic>;
    return result
        .map(
          (data) =>
              ExpandedDeviceApplication.fromJson(data as Map<String, dynamic>),
        )
        .toList();
  }

  Future<DeviceApplication> addDeviceApplication(
    String deviceId,
    String applicationId,
  ) async {
    final response = await client.post(
      '/devices/$deviceId/applications',
      data: {'application': applicationId},
    );

    return DeviceApplication.fromJson(response as Map<String, dynamic>);
  }

  Future<DeviceApplication> removeDeviceApplication(
    String deviceId,
    String applicationId,
  ) async {
    final response = await client.delete(
      '/devices/$deviceId/applications/$applicationId',
    );

    return DeviceApplication.fromJson(response as Map<String, dynamic>);
  }
}
