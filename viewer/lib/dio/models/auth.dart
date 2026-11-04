import 'user.dart';

class AuthResponse {
  final User user;
  final String token;
  final String sessionId;
  final String refreshToken;

  const AuthResponse({
    required this.user,
    required this.token,
    required this.sessionId,
    required this.refreshToken,
  });

  factory AuthResponse.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'token': String token,
        'session_id': String sessionId,
        'user': Map<String, dynamic> user,
        'refresh_token': String refreshToken,
      } =>
        AuthResponse(
          token: token,
          sessionId: sessionId,
          user: User.fromJson(user),
          refreshToken: refreshToken,
        ),
      _ => throw const FormatException("Failed to deserialize auth response"),
    };
  }
}
