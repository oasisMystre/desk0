import '../ffi/types/rdp.dart' show SseDesktopSize;

const kDefaultDeviceWidth = 1280;
const kDefaultDeviceHeight = 720;

class DeviceMetadataV1 {
  final SseDesktopSize size;

  const DeviceMetadataV1({required this.size});

  factory DeviceMetadataV1.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {'_version': 1, 'size': final Map<String, dynamic>? size} =>
        DeviceMetadataV1(
          size: size == null
              ? SseDesktopSize(
                  width: kDefaultDeviceWidth,
                  height: kDefaultDeviceHeight,
                )
              : SseDesktopSize(
                  width: (size['width'] as num).toInt().clamp(1, 65535),
                  height: (size['height'] as num).toInt().clamp(1, 65535),
                ),
        ),
      _ => throw const FormatException("Failed to deserialize device.metadata"),
    };
  }
}

class Device {
  bool? favorite;
  bool online = false;
  DeviceMetadataV1 metadata;
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

  String get name => username == null
      ? hostname == null
            ? "unknown device"
            : hostname!
      : username!;

  factory Device.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'os': String? os,
        'version': String? version,
        'favorite': bool? favorite,
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
              ? DeviceMetadataV1.fromJson({
                  "_version": 1,
                  "size": {
                      "width": kDefaultDeviceWidth,
                    "height": kDefaultDeviceHeight,
                  },
                })
              : DeviceMetadataV1.fromJson(metadata),
        ),
      _ => throw const FormatException('Failed to deserialize client'),
    };
  }
}

