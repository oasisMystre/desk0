import 'models/auth.dart';

import 'api_impl.dart';

class AuthApi extends ApiImpl {
  const AuthApi({required super.client});

  Future<void> emailSignUp(String email) async {
    await client.post(
      '/auth/email-signup',
      data: { 'email': email },
    );
  }

  Future<AuthResponse> verifyOTP(String otp) async {
    final response = await client.post('/auth/verify-otp', data: {'otp': otp});

    final result = response.data as Map<String, dynamic>;
    return AuthResponse.fromJson(result);
  }
}
