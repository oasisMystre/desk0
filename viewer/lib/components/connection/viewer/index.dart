import 'dart:math' show Point;
import 'dart:async' show StreamSubscription;

import 'package:get/get.dart' show Obx;
import 'package:flutter/material.dart' hide KeyboardListener;
import 'package:irondash_engine_context/irondash_engine_context.dart'
    show EngineContext;
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show RustStreamSink;

import 'mouse.dart' show MouseListener;
import 'keyboard.dart' show KeyboardListener;
import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../states/display_state.dart' show DisplayState;
import 'connection_loading.dart' show ConnectionOverlay, ConnectionPhase;
import '../../../ffi/types/rdp.dart'
    show
        SseDesktopSize,
        SseOutputEvent,
        SseOutputEvent_Resize,
        SseOutputEvent_Ignored,
        SseOutputEvent_Connected,
        SseOutputEvent_LoginComplete,
        SseOutputEvent_MonitorLayout,
        SseOutputEvent_PointerBitmap,
        SseOutputEvent_PointerDefault,
        SseOutputEvent_PointerHidden,
        SseOutputEvent_PointerPosition,
        SseOutputEvent_TerminatedError,
        SseOutputEvent_WindowingOrders,
        SseOutputEvent_AutoReconnected,
        SseOutputEvent_AutoReconnecting,
        SseOutputEvent_TerminatedSuccess,
        SseOutputEvent_ConnectionFailure;

class DeviceViewer extends StatefulWidget {
  final SseDesktopSize size;
  final ViewerHandle viewer;

  const DeviceViewer({super.key, required this.viewer, required this.size});

  @override
  DeviceViewerState createState() {
    return DeviceViewerState();
  }
}

class DeviceViewerState extends State<DeviceViewer> {
  int? textureId;
  String? connectionMessage;
  ConnectionPhase phase = ConnectionPhase.connecting;
  StreamSubscription<SseOutputEvent>? viewerSubscription;

  late final DisplayState displayState;

  @override
  void initState() {
    super.initState();
    final size = SseDesktopSize(
      width: DisplayState.alignDown(widget.size.width, 64),
      height: DisplayState.alignDown(widget.size.height, 2),
    );
    displayState = DisplayState(width: size.width, height: size.height);

    init();
  }

  Future<void> init() async {
    final engineHandle = await EngineContext.instance.getEngineHandle();
    final textureId = widget.viewer.createTexture(engineHandle: engineHandle);

    if (!mounted) {
      return;
    }

    setState(() {
      this.textureId = textureId;
    });

    final sink = RustStreamSink<SseOutputEvent>();
    widget.viewer.connect(events: sink);
    viewerSubscription = sink.stream.listen(
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

    super.dispose();
  }

  void onEvent(SseOutputEvent event) {
    switch (event) {
      case SseOutputEvent_Resize(:final width, :final height):
        displayState.resize(width, height);
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
        viewer: widget.viewer,
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
                    viewer: widget.viewer,
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
