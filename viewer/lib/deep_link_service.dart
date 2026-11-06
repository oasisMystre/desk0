import 'dart:async';

import 'package:app_links/app_links.dart';

extension DeepLinkUri on Uri {
  String? get route {
    if (scheme == 'desk0') return host;
    return pathSegments.firstOrNull;
  }
}

class DeepLinkService {
  static AppLinks appLinks = AppLinks();

  StreamSubscription<Uri>? subscription;

  Future<void> initialize(Future<void> Function(Uri uri) onConnection) async {
    final initialLink = await appLinks.getInitialLink();
    if (initialLink != null) {
      await onConnection(initialLink);
    }

    subscription = appLinks.uriLinkStream.listen(onConnection);
  }

  Future<void> dispose() async {
    await subscription?.cancel();
  }
}
