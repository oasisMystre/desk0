import 'package:dio/dio.dart';

abstract class ApiImpl {
  final Dio client;

  const ApiImpl({required this.client});
}
