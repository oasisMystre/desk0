import 'package:dio/dio.dart';
import 'package:get/get.dart';

import 'auth.dart';
import 'user.dart';
import 'devices.dart';
import 'attachment.dart';
import 'organization.dart';

export 'models/index.dart';

class Api extends GetxController {
  late final Dio client;

  final String baseUrl;
  String? userId, orgId, accessToken;

  late final AuthApi auth;
  late final UserApi user;
  late final DeviceApi device;
  late final AttachmentApi attachment;
  late final OrganizationApi organization;

  Api({required this.baseUrl, this.userId, this.orgId}) {
    client = Dio(
      BaseOptions(
        baseUrl: baseUrl,
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      ),
    );

    auth = AuthApi(client: client);
    user = UserApi(client: client);
    device = DeviceApi(client: client);
    attachment = AttachmentApi(client: client);
    organization = OrganizationApi(client: client);

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
}
