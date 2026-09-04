class Organization {
  final String id;
  final String? name;
  final String? user;
  final String createdAt;

  const Organization({
    required this.id,
    required this.user,
    required this.name,
    required this.createdAt,
  });

  factory Organization.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'name': String? name,
        'user': String? user,
        'created_at': String createdAt,
      } =>
        Organization(id: id, user: user, name: name, createdAt: createdAt),
      _ => throw const FormatException('Failed to load organization.'),
    };
  }
}
