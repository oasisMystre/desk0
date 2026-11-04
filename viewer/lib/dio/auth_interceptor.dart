import 'package:get/get_connect/http/src/status/http_status.dart'
    show HttpStatus;
import 'package:dio/dio.dart'
    show
        Dio,
        DioException,
        ErrorInterceptorHandler,
        Interceptor,
        RequestInterceptorHandler,
        RequestOptions;

class AuthInterceptor extends Interceptor {
  Future<void>? refreshFuture;

  final Dio dio;
  final String refreshPath;
  final Future<void> Function() clearSession;
  final Future<void> Function() refreshTokens;
  final Future<String?> Function() readAccessToken;

  AuthInterceptor({
    required this.dio,
    required this.refreshPath,
    required this.clearSession,
    required this.refreshTokens,
    required this.readAccessToken,
  });

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    if (options.extra['skipAuth'] == true) {
      return handler.next(options);
    }

    if (options.headers['Authorization'] == null) {
      final token = await readAccessToken();
      if (token != null && token.isNotEmpty) {
        options.headers['Authorization'] = 'Bearer $token';
      }
    }

    handler.next(options);
  }

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    final request = err.requestOptions;
    if (err.response?.statusCode != HttpStatus.unauthorized ||
        request.extra['retriedAfterRefresh'] == true ||
        request.path == refreshPath ||
        request.extra['skipAuth'] == true) {
      return handler.next(err);
    }

    try {
      final pendingRefresh = refreshFuture;
      if (pendingRefresh != null) {
        await pendingRefresh;
      } else {
        final refresh = refreshTokens();
        refreshFuture = refresh.then<void>((_) {});

        try {
          await refreshFuture;
        } finally {
          refreshFuture = null;
        }
      }

      final token = await readAccessToken();
      if (token == null || token.isEmpty) {
        throw StateError('Refresh did not provide an access token');
      }

      request
        ..headers['Authorization'] = 'Bearer $token'
        ..extra['retriedAfterRefresh'] = true;
    } catch (_) {
      await clearSession();
      return handler.next(err);
    }
  }
}
