import 'package:dio/dio.dart';
import 'package:get/get.dart';

import 'user.dart';
import 'devices.dart';
import 'attachment.dart';
import 'organization.dart';

export 'models/index.dart';

class Api extends GetxController {
  late final Dio client;

  final String baseUrl;
  String? _userId;
  String? _orgId;

  late final UserApi user;
  late final DeviceApi device;
  late final AttachmentApi attachment;
  late final OrganizationApi organization;

  Api({required this.baseUrl, this._userId, this._orgId}) {
    client = Dio(
      BaseOptions(
        baseUrl: baseUrl,
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json',
        },
      ),
    );

    user = UserApi(client: client);
    device = DeviceApi(client: client);
    attachment = AttachmentApi(client: client);
    organization = OrganizationApi(client: client);

    _updateHeaders();
  }

  String get websocketEndpoint {
    final endpoint = baseUrl.replaceFirst(RegExp(r'/+$'), '');
    return endpoint.endsWith('/ws') ? endpoint : '$endpoint/ws';
  }

  void _updateHeaders() {
    final headers = client.options.headers;

    if (_userId != null) {
      headers['x-user-id'] = _userId;
    } else {
      headers.remove('x-user-id');
    }

    if (_orgId != null) {
      headers['x-org-id'] = _orgId;
    } else {
      headers.remove('x-org-id');
    }
  }

  void updateUserId(String value) {
    _userId = value;
    _updateHeaders();
  }

  void updateOrganizationId(String value) {
    _orgId = value;
    _updateHeaders();
  }
}
