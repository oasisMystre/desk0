class User {
  final String id;
  final String? uid;
  final String createdAt;
  final String lastLoggedIn;

  const User({
    required this.id,
    required this.uid,
    required this.createdAt,
    required this.lastLoggedIn,
  });

  factory User.fromJson(Map<String, dynamic> json) {
    print(json);
    return switch (json) {
      {
        'id': String id,
        'uid': String? uid,
        'created_at': String createdAt,
        'last_logged_in': String lastLoggedIn,
      } =>
        User(
          id: id,
          uid: uid,
          createdAt: createdAt,
          lastLoggedIn: lastLoggedIn,
        ),
      _ => throw const FormatException('Failed to load user.'),
    };
  }
}
