extension FutureIgnoreError<T> on Future<T> {
  Future<T?> ignoreError() async {
    return await catchError(() => null);
  }
}

extension DeepLinkUri on Uri {
  String? get route {
    if (scheme == 'desk0') return host;
    return pathSegments.firstOrNull;
  }
}
