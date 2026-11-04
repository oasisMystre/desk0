import 'package:dio/dio.dart' show Options;

import 'api_impl.dart' show ApiImpl;
import 'models/index.dart' show AuthResponse, Session;

class AuthApi extends ApiImpl {
  const AuthApi({required super.client});

  Future<void> emailSignUp(String email) async {
    await client.post(
      '/auth/email-signup',
      data: {'email': email},
      options: Options(extra: {'skipAuth': true}),
    );
  }

  Future<AuthResponse> verifyOtp(String otp) async {
    final response = await client.post<Map<String, dynamic>>(
      '/auth/verify-otp',
      data: {'otp': otp},
      options: Options(extra: {'skipAuth': true}),
    );

    final result = response.data!;
    return AuthResponse.fromJson(result);
  }

  Future<List<Session>> getSessions() async {
    final response = await client.post<List<dynamic>>('/auth/sessions');

    final result = response.data!;
    return result
        .map((result) => Session.fromJson(result as Map<String, dynamic>))
        .toList();
  }

  Future<void> revokeSession(String session) async {
    await client.delete<List<dynamic>>('/auth/sessions/$session');
  }

  Future<void> logout() async {
    await client.post<List<dynamic>>('/auth/logout');
  }
}
