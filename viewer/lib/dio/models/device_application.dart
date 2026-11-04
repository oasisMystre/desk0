import 'application.dart' show ExpandedApplication;

class DeviceApplication {
  final DateTime createdAt;
  final String id, device, application, organization;

  const DeviceApplication({
    required this.id,
    required this.device,
    required this.application,
    required this.organization,
    required this.createdAt,
  });

  factory DeviceApplication.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'device': String device,
        'application': String application,
        'organization': String organization,
        'created_at': String createdAt,
      } =>
        DeviceApplication(
          id: id,
          device: device,
          application: application,
          organization: organization,
          createdAt: DateTime.parse(createdAt),
        ),
      _ => throw const FormatException(
        "Failed to deserialize device application",
      ),
    };
  }
}

class ExpandedDeviceApplication {
  final String id, device;
  final DateTime createdAt;
  final ExpandedApplication application;

  const ExpandedDeviceApplication({
    required this.id,
    required this.device,
    required this.application,
    required this.createdAt,
  });

  factory ExpandedDeviceApplication.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'device': String device,
        'created_at': String createdAt,
        'application': Map<String, dynamic> application,
      } =>
        ExpandedDeviceApplication(
          id: id,
          device: device,
          createdAt: DateTime.parse(createdAt),
          application: ExpandedApplication.fromJson(application),
        ),
      _ => throw const FormatException(
        "Failed to deserialize device application",
      ),
    };
  }
}
