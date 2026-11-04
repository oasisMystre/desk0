import 'package:dio/dio.dart' show Dio;

abstract class ApiImpl {
  final Dio client;

  const ApiImpl({required this.client});
}
