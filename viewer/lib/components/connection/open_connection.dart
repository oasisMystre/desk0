import 'package:flutter/cupertino.dart';

import 'viewer/index.dart' show DeviceViewer;
import 'terminal/index.dart' show TerminalViewer;
import '../../ffi/flutter.dart' show ViewerHandle;
import '../../ffi/types.dart' show SseDesktopSize;
import '../../states/display_state.dart' show DisplayState;

enum ConnectionType { terminal, desktop }

class OpenConnection extends StatefulWidget {
  final SseDesktopSize size;
  final String endpoint, token;
  final ConnectionType connectionType;

  const OpenConnection({
    super.key,
    required this.size,
    required this.token,
    required this.endpoint,
    required this.connectionType,
  });

  @override
  OpenConnectionState createState() {
    return OpenConnectionState();
  }
}

class OpenConnectionState extends State<OpenConnection> {
  late final SseDesktopSize size;
  late final ViewerHandle viewer;

  @override
  void initState() {
    final uri = Uri.parse(widget.endpoint);
    size = SseDesktopSize(
      width: DisplayState.alignDown(widget.size.width, 64),
      height: DisplayState.alignDown(widget.size.height, 2),
    );

    viewer = ViewerHandle(
      size: size,
      token: widget.token,
      endpoint: uri.path == '/ws' ? widget.endpoint : '${uri.host}/ws',
    );

    viewer.connect();

    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    switch (widget.connectionType) {
      case ConnectionType.terminal:
        return TerminalViewer(viewer: viewer);
      case ConnectionType.desktop:
        return DeviceViewer(viewer: viewer, size: size);
    }
  }
}
