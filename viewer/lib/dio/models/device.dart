class Device {
  final String id;
  final String? os;
  final String? hostname;
  final String? username;
  final String? version;
  final String organization;
  final String lastLoggedIn;
  final String createdAt;
  final bool online = false;

  Device({
    required this.id,
    required this.os,
    required this.hostname,
    required this.username,
    required this.version,
    required this.organization,
    required this.lastLoggedIn,
    required this.createdAt,
  });

  factory Device.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'os': String? os,
        'hostname': String? hostname,
        'username': String? username,
        'version': String? version,
        'organization': String organization,
        'last_logged_in': String lastLoggedIn,
        'created_at': String createdAt,
      } =>
        Device(
          id: id,
          os: os,
          hostname: hostname,
          username: username,
          version: version,
          organization: organization,
          lastLoggedIn: lastLoggedIn,
          createdAt: createdAt,
        ),
      _ => throw const FormatException('Failed to load client.'),
    };
  }
}
