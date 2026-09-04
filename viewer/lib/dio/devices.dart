import 'apl_impl.dart';
import 'models/device.dart';

class DeviceApi extends ApiImpl {
  const DeviceApi({required super.client});

  Future<List<Device>> getDevices() async {
    final response = await client.get('/devices');
    if (response.statusCode == 200) {
      final result = response.data as List<Map<String, dynamic>>;
      return result.map((data) => Device.fromJson(data)).toList();
    } else {
      throw Exception(response.data);
    }
  }

  Future<Device> deleteDevicet(String id, bool uploaded) async {
    final response = await client.delete(
      '/devices/$id',
      data: {'uploaded': uploaded},
    );

    if (response.statusCode == 200) {
      final result = response.data as Map<String, dynamic>;
      return Device.fromJson(result);
    } else {
      throw Exception(response.data);
    }
  }
}
