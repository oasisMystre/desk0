import 'dart:async';

import 'package:app_links/app_links.dart';

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
