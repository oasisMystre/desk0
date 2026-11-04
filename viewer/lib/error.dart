import 'dart:developer';

import 'package:dio/dio.dart';
import 'package:flutter/material.dart';

Future<T?> handleHttpErrorFromContext<T>(
  BuildContext context,
  Future<T> Function() func,
  String defaultErrorMessage,
) async {
  try {
    return await func();
  } on DioException catch (error, stackTrace) {
    log(
      'HTTP request failed',
      name: 'desk0.http',
      error: error,
      stackTrace: stackTrace,
    );

    final data = error.response?.data;
    final message = data is Map ? data['message'] : null;

    showError(
      context,
      message is String && message.isNotEmpty ? message : defaultErrorMessage,
    );

    return null;
  } catch (error, stackTrace) {
    log(
      'Unexpected request error',
      name: 'desk0.http',
      error: error,
      stackTrace: stackTrace,
    );

    showError(context, defaultErrorMessage);
    return null;
  }
}

void showError(BuildContext context, String message) {
  if (!context.mounted) return;

  ScaffoldMessenger.of(context)
    ..hideCurrentSnackBar()
    ..showSnackBar(SnackBar(content: Text(message)));
}
