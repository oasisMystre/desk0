import 'dart:math' show Point;
import 'dart:async' show StreamSubscription;

import 'package:flutter/foundation.dart';
import 'package:get/get.dart' show Obx;
import 'package:irondash_engine_context/irondash_engine_context.dart'
    show EngineContext;
import 'package:flutter/material.dart'
    show CircularProgressIndicator, Positioned, Stack, StackFit;

import 'package:flutter/cupertino.dart'
    show
        AspectRatio,
        BuildContext,
        Center,
        LayoutBuilder,
        State,
        StatefulWidget,
        Texture,
        Widget;

import 'mouse.dart' show MouseListener;
import 'keyboard.dart' show KeyboardListener;
import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../states/display_state.dart' show DisplayState;
import 'connection_loading.dart' show ConnectionOverlay, ConnectionPhase;
import '../../../ffi/types.dart'
    show
        SseDesktopSize,
        SseOutputEvent,
        SseOutputEvent_AutoReconnected,
        SseOutputEvent_AutoReconnecting,
        SseOutputEvent_Clipboard,
        SseOutputEvent_Connected,
        SseOutputEvent_ConnectionFailure,
        SseOutputEvent_Ignored,
        SseOutputEvent_LoginComplete,
        SseOutputEvent_MonitorLayout,
        SseOutputEvent_PointerBitmap,
        SseOutputEvent_PointerDefault,
        SseOutputEvent_PointerHidden,
        SseOutputEvent_PointerPosition,
        SseOutputEvent_Resize,
        SseOutputEvent_TerminatedError,
        SseOutputEvent_TerminatedSuccess,
        SseOutputEvent_WindowingOrders;

class DeviceViewer extends StatefulWidget {
  final String token;
  final String endpoint;
  final SseDesktopSize size;

  const DeviceViewer({
    super.key,
    required this.endpoint,
    required this.token,
    required this.size,
  });

  @override
  DeviceViewerState createState() {
    return DeviceViewerState();
  }
}

class DeviceViewerState extends State<DeviceViewer> {
  int? textureId;
  late final DisplayState displayState;
  StreamSubscription<SseOutputEvent>? viewerSubscription;

  late final ViewerHandle viewer;

  String? connectionMessage;
  ConnectionPhase phase = ConnectionPhase.connecting;

  @override
  void initState() {
    super.initState();
    final size = SseDesktopSize(
      width: DisplayState.alignDown(widget.size.width, 64),
      height: DisplayState.alignDown(widget.size.height, 2),
    );
    displayState = DisplayState(width: size.width, height: size.height);

    final uri = Uri.parse(widget.endpoint);

    viewer = ViewerHandle(
      size: size,
      token: widget.token,
      endpoint: uri.path == '/ws' ? widget.endpoint : '${uri.host}/ws',
    );

    init();
  }

  Future<void> init() async {
    final engineHandle = await EngineContext.instance.getEngineHandle();
    final textureId = viewer.createTexture(engineHandle: engineHandle);

    if (!mounted) {
      return;
    }

    setState(() {
      this.textureId = textureId;
    });

    viewerSubscription = viewer.connect().listen(
      onEvent,
      onError: (Object error, _) {
        setPhase(ConnectionPhase.failed, error.toString());
      },
    );
  }

  void setPhase(ConnectionPhase nextPhase, [String? message]) {
    if (!mounted || (phase == nextPhase && connectionMessage == message)) {
      return;
    }

    setState(() {
      phase = nextPhase;
      connectionMessage = message;
    });
  }

  @override
  void dispose() {
    viewerSubscription?.cancel();
    viewer.dispose();

    super.dispose();
  }

  void onEvent(SseOutputEvent event) {
    switch (event) {
      case SseOutputEvent_Resize(:final width, :final height):
        displayState.resize(width, height);
        break;
      case SseOutputEvent_PointerHidden():
        debugPrint("pointer hide");
        displayState.hideCursor();

        break;
      case SseOutputEvent_PointerDefault():
        debugPrint("pointer default");
        displayState.defaultCursor();
        break;
      case SseOutputEvent_PointerBitmap(
        :final width,
        :final height,
        :final bitmapData,
        :final hotspotX,
        :final hotspotY,
      ):
        debugPrint("pointer bitmap");
        debugPrint(
          'Received cursor: ${width}x$height, ${bitmapData.length} bytes',
        );
        displayState.onCursorBitmap(
          width: width,
          height: height,
          bitmapData: bitmapData,
          hotspotX: hotspotX,
          hotspotY: hotspotY,
        );
        break;
      case SseOutputEvent_TerminatedError(:final field0):
        setPhase(ConnectionPhase.failed, field0);
        break;
      case SseOutputEvent_TerminatedSuccess():
        setPhase(ConnectionPhase.failed, 'Connection closed');
        break;
      case SseOutputEvent_ConnectionFailure(:final field0):
        setPhase(ConnectionPhase.failed, field0);
        break;
      case SseOutputEvent_PointerPosition():
        break;
      case SseOutputEvent_Connected():
        setPhase(ConnectionPhase.connected);
        break;
      case SseOutputEvent_AutoReconnecting(
        :final attempt,
        :final maximumAttempts,
      ):
        setPhase(
          ConnectionPhase.reconnecting,
          'Reconnecting ($attempt/$maximumAttempts)…',
        );
        break;
      case SseOutputEvent_AutoReconnected():
        setPhase(ConnectionPhase.connected);
        break;
      case SseOutputEvent_LoginComplete():
        break;
      case SseOutputEvent_MonitorLayout():
        break;
      case SseOutputEvent_Clipboard():
        break;
      case SseOutputEvent_WindowingOrders():
        break;
      case SseOutputEvent_Ignored():
        break;
    }
  }

  @override
  Widget build(BuildContext context) {
    final id = textureId;
    if (id == null) {
      return const Center(child: CircularProgressIndicator());
    }

    final content = Obx(() {
      final cursor = displayState.cursor.value;
      final width = displayState.width.value;
      final height = displayState.height.value;

      return KeyboardListener(
        viewer: viewer,
        child: LayoutBuilder(
          builder: (context, constraints) {
            final frameAspectRatio = width / height;

            return AspectRatio(
              aspectRatio: frameAspectRatio,
              child: LayoutBuilder(
                builder: (context, constraints) {
                  final scaleX = width / constraints.maxWidth;
                  final scaleY = height / constraints.maxHeight;

                  return MouseListener(
                    cursor: cursor,
                    viewer: viewer,
                    scale: Point(scaleX, scaleY),
                    child: Texture(textureId: id),
                  );
                },
              ),
            );
          },
        ),
      );
    });

    return Stack(
      fit: StackFit.expand,
      children: [
        content,
        if (phase != ConnectionPhase.connected)
          Positioned.fill(
            child: ConnectionOverlay(phase: phase, message: connectionMessage),
          ),
      ],
    );
  }
}
