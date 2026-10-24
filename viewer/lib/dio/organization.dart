import 'api_impl.dart';
import 'models/organization.dart';

class OrganizationApi extends ApiImpl {
  const OrganizationApi({required super.client});

  Future<List<Organization>> getOrganizations(String? name) async {
    final response = await client.post('/organizations', data: {'name': name});

    final result = response.data as List<Map<String, dynamic>>;
    return result.map((data) => Organization.fromJson(data)).toList();
  }

  Future<Organization> createOrganization(String? name) async {
    final response = await client.post('/organizations', data: {'name': name});

    final result = response.data as Map<String, dynamic>;
    return Organization.fromJson(result);
  }

  Future<String> createConnectionToken(String deviceId) async {
    final response = await client.post('/organizations/token/$deviceId');

    final result = response.data as Map<String, dynamic>;
    return result['token'];
  }
}
