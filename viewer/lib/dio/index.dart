import 'package:get/get.dart' show GetxController;
import 'package:dio/dio.dart' show Dio, BaseOptions;
import 'package:shared_preferences/shared_preferences.dart'
    show SharedPreferences;

import 'auth.dart' show AuthApi;
import 'user.dart' show UserApi;
import 'agent.dart' show AgentApi;
import 'devices.dart' show DeviceApi;
import 'attachment.dart' show AttachmentApi;
import 'application.dart' show ApplicationApi;
import 'organization.dart' show OrganizationApi;
import 'auth_interceptor.dart' show AuthInterceptor;
import 'device_application.dart' show DeviceApplicationApi;

export 'models/index.dart';

class Api extends GetxController {
  String? userId, orgId, accessToken;

  final String baseUrl;
  late final Dio client;
  late final AuthApi auth;
  late final UserApi user;
  late final AgentApi agent;
  late final DeviceApi device;
  late final AttachmentApi attachment;
  late final ApplicationApi application;
  late final OrganizationApi organization;
  late final DeviceApplicationApi deviceApplication;

  late final Dio refreshClient;
  final SharedPreferences sharedPreferences;

  Api({
    this.orgId,
    this.userId,
    required this.baseUrl,
    required this.sharedPreferences,
  }) {
    final options = BaseOptions(
      baseUrl: baseUrl,
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
    );
    refreshClient = Dio(options);
    client = Dio(options);

    client.interceptors.add(
      AuthInterceptor(
        dio: client,
        clearSession: clearSession,
        refreshTokens: refreshTokens,
        readAccessToken: readAccessToken,
        refreshPath: '/auth/refresh',
      ),
    );

    auth = AuthApi(client: client);
    user = UserApi(client: client);
    agent = AgentApi(client: client);
    device = DeviceApi(client: client);
    attachment = AttachmentApi(client: client);
    application = ApplicationApi(client: client);
    organization = OrganizationApi(client: client);
    deviceApplication = DeviceApplicationApi(client: client);

    updateHeaders();
  }

  void updateHeaders() {
    final headers = client.options.headers;

    if (userId != null) {
      headers['x-user-id'] = userId;
    } else {
      headers.remove('x-user-id');
    }

    if (orgId != null) {
      headers['x-org-id'] = orgId;
    } else {
      headers.remove('x-org-id');
    }

    if (accessToken != null) {
      headers['Authorization'] = 'Bearer $accessToken';
    } else {
      headers.remove('Authorization');
    }
  }

  void updateUserId(String? value) {
    userId = value;
    updateHeaders();
  }

  void updateOrganizationId(String? value) {
    orgId = value;
    updateHeaders();
  }

  void updateAccessToken(String? value) {
    accessToken = value;
    updateHeaders();
  }

  Future<String?> refreshTokens() async {
    final refreshToken = sharedPreferences.getString('refreshToken');
    if (refreshToken == null) return null;
    final response = await refreshClient.post<Map<String, dynamic>>(
      '/auth/refresh',
      data: {'refresh_token': refreshToken},
    );

    final data = response.data!;
    final accessToken = data['token'] as String;
    final newRefreshToken = data['refresh_token'] as String;

    sharedPreferences.setString('token', accessToken);
    sharedPreferences.setString('refresh_token', newRefreshToken);

    return accessToken;
  }

  Future<String?> readAccessToken() async {
    final accessToken = sharedPreferences.getString('accessToken');
    return accessToken;
  }

  Future<void> clearSession() async {
    sharedPreferences.remove('accessToken');
    sharedPreferences.remove('refreshToken');
  }
}
