import 'dart:io' show HttpStatus;

import 'package:get/get.dart';
import 'package:dio/dio.dart' show DioException;
import 'package:shared_preferences/shared_preferences.dart'
    show SharedPreferences;

import '../../dio/index.dart' show Api, User, AuthResponse;

class AuthState extends GetxController {
  static const accessTokenCacheKey = "accessToken";

  late final Api api;
  final SharedPreferences sharedPreferences;

  RxBool ready = false.obs;
  Rx<User?> user = null.obs;

  AuthState(this.sharedPreferences) {
    api = Get.find<Api>();
  }

  Future<void> init() async {
    final cachedToken = sharedPreferences.getString(accessTokenCacheKey);

    try {
      if (cachedToken != null) {
        api.updateAccessToken(cachedToken);
        final apiUser = await api.user.getUser();
        user.value = apiUser;
      }
    } on DioException catch (error) {
      if (error.response?.statusCode == HttpStatus.unauthorized) {
        api.updateAccessToken(null);
        await sharedPreferences.remove(accessTokenCacheKey);
      } else {
        rethrow;
      }
    } finally {
      ready.value = true;
    }
  }

  Future<void> loginWithEmail(String email) async {
    return await api.auth.emailSignUp(email);
  }

  Future<AuthResponse> verifyOTP(String otp) async {
    final response = await api.auth.verifyOTP(otp);
    final apiUser = response.user;
    final token = response.token;

    user.value = apiUser;
    api.updateAccessToken(token);
    await sharedPreferences.setString(accessTokenCacheKey, token);

    return response;
  }
}
