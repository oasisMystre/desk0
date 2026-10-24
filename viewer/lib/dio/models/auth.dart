import 'user.dart';

class AuthResponse {
  final User user;
  final String token;

  const AuthResponse({required this.user, required this.token});

  factory AuthResponse.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {'user': dynamic user, 'token': String token} => AuthResponse(
        token: token,
        user: user,
      ),
      _ => throw const FormatException("Failed to deserialize auth response"),
    };
  }
}
