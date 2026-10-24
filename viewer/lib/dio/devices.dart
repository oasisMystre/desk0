import 'api_impl.dart';
import 'models/device.dart';

class DeviceApi extends ApiImpl {
  const DeviceApi({required super.client});

  Future<List<Device>> getDevices() async {
    final response = await client.get('/devices');
    final result = response.data as List<dynamic>;
    return result
        .map((data) => Device.fromJson(data as Map<String, dynamic>))
        .toList();
  }

  Future<Device> deleteDevice(String id) async {
    final response = await client.delete('/devices/$id');

    final result = response.data as Map<String, dynamic>;
    return Device.fromJson(result);
  }
}
