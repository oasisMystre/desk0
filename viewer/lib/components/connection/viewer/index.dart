import 'dart:math' show Point;
import 'dart:async' show StreamSubscription;

import 'package:get/get.dart' show Obx;
import 'package:texture_rgba_renderer/texture_rgba_renderer.dart'
    show TextureRgbaRenderer;
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
        Widget,
        WidgetsBinding;

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
        SseOutputEvent_DesktopUpdate,
        SseOutputEvent_Ignored,
        SseOutputEvent_Image,
        SseOutputEvent_LoginComplete,
        SseOutputEvent_MonitorLayout,
        SseOutputEvent_PointerBitmap,
        SseOutputEvent_PointerDefault,
        SseOutputEvent_PointerHidden,
        SseOutputEvent_PointerPosition,
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
  late final TextureRgbaRenderer textureRgbaRenderer;
  StreamSubscription<SseOutputEvent>? viewerSubscription;

  late final ViewerHandle viewer;

  String? connectionMessage;
  ConnectionPhase phase = ConnectionPhase.connecting;

  @override
  void initState() {
    super.initState();
    textureRgbaRenderer = TextureRgbaRenderer();
    displayState = DisplayState(
      frameBufferWidth: widget.size.width,
      frameBufferHeight: widget.size.height,
      textureRgbaRenderer: textureRgbaRenderer,
    );

    final uri = Uri.parse(widget.endpoint);

    viewer = ViewerHandle(
      size: widget.size,
      token: widget.token,
      endpoint: uri.path == '/ws' ? widget.endpoint : '${uri.host}/ws',
    );

    init();
  }

  Future<void> init() async {
    textureId = await textureRgbaRenderer.createTexture(0);

    if (!mounted) {
      return;
    }

    setState(() {});

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
    textureRgbaRenderer.closeTexture(0);

    super.dispose();
  }

  void onEvent(SseOutputEvent event) {
    switch (event) {
      case SseOutputEvent_Image(:final width, :final height, :final buffer):
        displayState.onImage(width: width, height: height, buffer: buffer);
        break;
      case SseOutputEvent_PointerHidden():
        displayState.hideCursor();
        break;
      case SseOutputEvent_PointerDefault():
        displayState.defaultCursor();
        break;
      case SseOutputEvent_PointerBitmap(
        :final width,
        :final height,
        :final bitmapData,
        :final hotspotX,
        :final hotspotY,
      ):
        displayState.onCursorBitmap(
          width: width,
          height: height,
          bitmapData: bitmapData,
          hotspotX: hotspotX,
          hotspotY: hotspotY,
        );
        break;
      case SseOutputEvent_DesktopUpdate(
        :final width,
        :final height,
        :final buffer,
        :final region,
      ):
        displayState.onDesktopUpdate(
          width: width,
          height: height,
          buffer: buffer,
          region: region,
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
      final frameWidth = displayState.frameBufferWidth.value;
      final frameHeight = displayState.frameBufferHeight.value;

      return KeyboardListener(
        viewer: viewer,
        child: LayoutBuilder(
          builder: (context, constraints) {
            if (phase == ConnectionPhase.connected) {
              WidgetsBinding.instance.addPostFrameCallback((_) {
                if (mounted) {
                  displayState.onResize(viewer, context, constraints);
                }
              });
            }

            final frameAspectRatio = frameWidth / frameHeight;

            return Center(
              child: AspectRatio(
                aspectRatio: frameAspectRatio,
                child: LayoutBuilder(
                  builder: (context, constraints) {
                    final scaleX = frameWidth / constraints.maxWidth;
                    final scaleY = frameHeight / constraints.maxHeight;

                    return MouseListener(
                      cursor: cursor,
                      viewer: viewer,
                      scale: Point(scaleX, scaleY),
                      child: Texture(textureId: id),
                    );
                  },
                ),
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
