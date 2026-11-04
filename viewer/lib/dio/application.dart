import 'package:dio_cache_interceptor/dio_cache_interceptor.dart'
    show CacheOptionExtension;

import 'api_impl.dart' show ApiImpl;
import 'cache_options.dart' show cacheOptions;
import 'models/application.dart'
    show Application, CreateApplication, ExpandedApplication;

class ApplicationApi extends ApiImpl {
  ApplicationApi({required super.client});

  Future<ExpandedApplication> createApplication(CreateApplication application) async {
    final data = application.toFormData();
    final response = await client.post('/applications', data: data);
    return ExpandedApplication.fromJson(response.data as Map<String, dynamic>);
  }

  Future<List<ExpandedApplication>> getApplications() async {
    final response = await client.get(
      '/applications',
      options: (await cacheOptions).toOptions(),
    );
    final result = response.data as List<dynamic>;
    return result
        .map(
          (data) => ExpandedApplication.fromJson(data as Map<String, dynamic>),
        )
        .toList();
  }

  Future<ExpandedApplication> getApplication(String id) async {
    final response = await client.get(
      '/applications/$id',
      options: (await cacheOptions).toOptions(),
    );
    return ExpandedApplication.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Application> deleteApplication(String id) async {
    final response = await client.delete('/applications/$id');
    return Application.fromJson(response.data as Map<String, dynamic>);
  }

 
}
