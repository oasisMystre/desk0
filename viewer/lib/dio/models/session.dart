class Session {
  final String id;
  final bool isCurrent;
  final String? userAgent, ipAddress, createdAt, expiresAt, lastUsedAt;

  const Session({
    required this.id,
    required this.isCurrent,
    required this.userAgent,
    required this.ipAddress,
    required this.createdAt,
    required this.expiresAt,
    required this.lastUsedAt,
  });

  factory Session.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'is_current': bool isCurrent,
        'user_agent': String? userAgent,
        'ip_address': String? ipAddress,
        'created_at': String? createdAt,
        'expires_at': String? expiresAt,
        'last_used_at': String? lastUsedAt,
      } =>
        Session(
          id: id,
          isCurrent: isCurrent,
          userAgent: userAgent,
          ipAddress: ipAddress,
          createdAt: createdAt,
          expiresAt: expiresAt,
          lastUsedAt: lastUsedAt,
        ),
      _ => throw const FormatException('Failed to deserialize session'),
    };
  }
}
