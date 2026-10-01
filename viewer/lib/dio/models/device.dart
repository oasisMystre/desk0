import 'package:flutter/painting.dart' show Size;

class DeviceMetadataV1 {
  final Size? size;

  const DeviceMetadataV1({required this.size});

  factory DeviceMetadataV1.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {'_version': 1, 'size': final Map<String, dynamic>? size} =>
        DeviceMetadataV1(
          size: size == null
              ? null
              : Size(
                  (size['width'] as num).toDouble(),
                  (size['height'] as num).toDouble(),
                ),
        ),
      _ => throw const FormatException("Failed to deserialize device.metadata"),
    };
  }
}

class Device {
  bool favorite;
  bool online = false;
  DeviceMetadataV1? metadata;
  String? hostname, username, version, os;
  String id, organization, lastLoggedIn, createdAt;

  Device({
    required this.id,
    required this.os,
    required this.favorite,
    required this.metadata,
    required this.hostname,
    required this.username,
    required this.version,
    required this.createdAt,
    required this.organization,
    required this.lastLoggedIn,
  });

  factory Device.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'os': String? os,
        'version': String? version,
        'favorite': bool favorite,
        'hostname': String? hostname,
        'username': String? username,
        'created_at': String createdAt,
        'organization': String organization,
        'last_logged_in': String lastLoggedIn,
        'metadata': Map<String, dynamic>? metadata,
      } =>
        Device(
          id: id,
          os: os,
          favorite: favorite,
          hostname: hostname,
          username: username,
          version: version,
          createdAt: createdAt,
          organization: organization,
          lastLoggedIn: lastLoggedIn,
          metadata: metadata == null
              ? null
              : DeviceMetadataV1.fromJson(metadata),
        ),
      _ => throw const FormatException('Failed to deserialize client'),
    };
  }
}
