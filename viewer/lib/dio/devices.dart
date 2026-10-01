import 'apl_impl.dart';
import 'models/device.dart';

class DeviceApi extends ApiImpl {
  const DeviceApi({required super.client});

  Future<List<Device>> getDevices() async {
    final response = await client.get('/devices');
    if (response.statusCode == 200) {
      final result = response.data as List<dynamic>;
      return result
          .map((data) => Device.fromJson(data as Map<String, dynamic>))
          .toList();
    } else {
      throw Exception(response.data);
    }
  }

  Future<Device> deleteDevice(String id) async {
    final response = await client.delete('/devices/$id');

    if (response.statusCode == 200) {
      final result = response.data as Map<String, dynamic>;
      return Device.fromJson(result);
    } else {
      throw Exception(response.data);
    }
  }
}
