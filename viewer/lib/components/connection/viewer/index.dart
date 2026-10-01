import 'dart:math' show Point;
import 'dart:async' show StreamSubscription;

import 'package:flutter/services.dart' show MouseCursor, SystemMouseCursors;
import 'package:flutter_custom_cursor/cursor_manager.dart'
    show CursorManager, CursorData;
import 'package:flutter_custom_cursor/flutter_custom_cursor.dart'
    show FlutterCustomMemoryImageCursor;
import 'package:texture_rgba_renderer/texture_rgba_renderer.dart'
    show TextureRgbaRenderer;
import 'package:flutter/cupertino.dart'
    show
        Size,
        State,
        StatefulWidget,
        BuildContext,
        Widget,
        Center,
        Texture,
        LayoutBuilder,
        AspectRatio;

import 'mouse.dart' show MouseListener;
import 'keyboard.dart' show KeyboardListener;
import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../ffi/types.dart'
    show
        SseDesktopSize,
        SseOutputEvent,
        SseOutputEvent_ConnectionFailure,
        SseOutputEvent_Image,
        SseOutputEvent_PointerBitmap,
        SseOutputEvent_PointerDefault,
        SseOutputEvent_PointerHidden,
        SseOutputEvent_PointerPosition,
        SseOutputEvent_TerminatedError,
        SseOutputEvent_TerminatedSuccess;

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
  late final TextureRgbaRenderer textureRgbaRenderer;
  StreamSubscription<SseOutputEvent>? viewerSubscription;

  late final ViewerHandle viewer;

  Size frameSize = const Size(1280, 720);
  MouseCursor cursor = SystemMouseCursors.none;

  @override
  void initState() {
    super.initState();
    textureRgbaRenderer = TextureRgbaRenderer();
    frameSize = Size(
      widget.size.width.toDouble(),
      widget.size.height.toDouble(),
    );

    viewer = ViewerHandle(endpoint: widget.endpoint, token: widget.token);

    init();
  }

  Future<void> init() async {
    textureId = await textureRgbaRenderer.createTexture(0);

    if (!mounted) {
      return;
    }

    setState(() {});

    viewerSubscription = viewer.connect().listen(onEvent, onError: (error) {});
  }

  @override
  void dispose() {
    viewerSubscription?.cancel();
    viewer.dispose();
    textureRgbaRenderer.closeTexture(0);

    super.dispose();
  }

  Future<void> onEvent(SseOutputEvent event) async {
    switch (event) {
      case SseOutputEvent_Image(:final width, :final height, :final buffer):
        final bytes = buffer.buffer.asUint8List(
          buffer.offsetInBytes,
          buffer.lengthInBytes,
        );

        textureRgbaRenderer.onRgba(0, bytes, height, width, 0);

        if (frameSize.width != width || frameSize.height != height) {
          setState(() {
            frameSize = Size(width.toDouble(), height.toDouble());
          });
        }
        break;

      case SseOutputEvent_PointerHidden():
        setState(() {
          cursor = SystemMouseCursors.none;
        });
        break;
      case SseOutputEvent_PointerDefault():
        setState(() {
          cursor = SystemMouseCursors.basic;
        });
        break;
      case SseOutputEvent_PointerBitmap(
        :final width,
        :final height,
        :final bitmapData,
        :final hotspotX,
        :final hotspotY,
      ):
        final cursorName = await CursorManager.instance.registerCursor(
          CursorData()
            ..buffer = bitmapData
            ..height = height
            ..width = width
            ..hotX = hotspotX.toDouble()
            ..hotY = hotspotY.toDouble(),
        );

        if (!mounted) return;

        setState(() {
          cursor = FlutterCustomMemoryImageCursor(key: cursorName);
        });
        break;

      case SseOutputEvent_TerminatedError():
        break;
      case SseOutputEvent_TerminatedSuccess():
        break;
      case SseOutputEvent_ConnectionFailure():
        break;
      case SseOutputEvent_PointerPosition():
        break;
    }
  }

  @override
  Widget build(BuildContext context) {
    final id = textureId;
    if (id == null) {
      return const Center();
    }

    return KeyboardListener(
      viewer: viewer,
      child: AspectRatio(
        aspectRatio: frameSize.width / frameSize.height,
        child: LayoutBuilder(
          builder: (context, box) {
            final scaleX = frameSize.width / box.maxWidth;
            final scaleY = frameSize.height / box.maxHeight;

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
  }
}
