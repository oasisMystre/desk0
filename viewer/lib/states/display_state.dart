import 'dart:typed_data' show Uint8List, Uint32List;

import 'package:desk0/ffi/flutter.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart' show GetxController, Rx, RxT;
import 'package:flutter_custom_cursor/cursor_manager.dart'
    show CursorManager, CursorData;
import 'package:flutter_custom_cursor/flutter_custom_cursor.dart'
    show FlutterCustomMemoryImageCursor;
import 'package:texture_rgba_renderer/texture_rgba_renderer.dart'
    show TextureRgbaRenderer;

import '../../../ffi/types.dart' show SseInclusizeRectangle;

class DisplayState extends GetxController {
  Uint32List? frameBuffer;
  int? textureWidth, textureHeight;

  final Rx<int> frameBufferWidth = 0.obs, frameBufferHeight = 0.obs;

  final TextureRgbaRenderer textureRgbaRenderer;
  final Rx<MouseCursor> cursor = SystemMouseCursors.none.obs;

  DisplayState({
    required this.textureRgbaRenderer,
    required int frameBufferWidth,
    required int frameBufferHeight,
  }) {
    this.frameBufferWidth.value = frameBufferWidth;
    this.frameBufferHeight.value = frameBufferHeight;
  }

  Future<void> onDesktopUpdate({
    required int width,
    required int height,
    required Uint32List buffer,
    required SseInclusizeRectangle region,
  }) async {
    if (frameBuffer == null ||
        frameBufferWidth.value != width ||
        frameBufferHeight.value != height) {
      updateFrameSize(width, height);
      frameBuffer = Uint32List(width * height);
    }

    final destination = frameBuffer!;

    final regionWidth = region.width;
    final regionHeight = region.height;

    for (var y = 0; y < regionHeight; y++) {
      final sourceStart = y * regionWidth;
      final destinationStart = (region.top + y) * width + region.left;

      for (var x = 0; x < regionWidth; x++) {
        destination[destinationStart + x] =
            buffer[sourceStart + x] | 0xFF000000;
      }
    }

    final bytes = destination.buffer.asUint8List(
      destination.offsetInBytes,
      destination.lengthInBytes,
    );

    await textureRgbaRenderer.onRgba(0, bytes, height, width, 64);
  }

  void onImage({
    required int width,
    required int height,
    required Uint32List buffer,
  }) {
    updateFrameSize(width, height);

    final pixels = Uint32List(buffer.length);

    for (var i = 0; i < buffer.length; i++) {
      pixels[i] = buffer[i] | 0xFF000000;
    }

    final bytes = pixels.buffer.asUint8List();

    textureRgbaRenderer.onRgba(0, bytes, height, width, 0);
  }

  void updateFrameSize(int width, int height) {
    if (frameBufferWidth.value != width) {
      frameBufferWidth.value = width;
    }
    if (frameBufferHeight.value != height) {
      frameBufferHeight.value = height;
    }
  }

  void onCursorBitmap({
    required int width,
    required int height,
    required Uint8List bitmapData,
    required int hotspotX,
    required int hotspotY,
  }) async {
    final cursorName = await CursorManager.instance.registerCursor(
      CursorData()
        ..buffer = bitmapData
        ..height = height
        ..width = width
        ..hotX = hotspotX.toDouble()
        ..hotY = hotspotY.toDouble(),
    );

    cursor.value = FlutterCustomMemoryImageCursor(key: cursorName);
  }

  void hideCursor() {
    cursor.value = SystemMouseCursors.none;
  }

  void defaultCursor() {
    cursor.value = SystemMouseCursors.basic;
  }

  void onResize(
    ViewerHandle viewer,
    BuildContext context,
    BoxConstraints constraints,
  ) {
    final dpr = View.of(context).devicePixelRatio;
    int alignDown(int v, int a) => (v ~/ a) * a;

    final width = alignDown(constraints.maxWidth.round(), 16);
    final height = alignDown(constraints.maxHeight.round(), 2);

    if (width <= 0 || height <= 0) {
      return;
    }

    if (textureWidth == width && textureHeight == height) {
      return;
    }

    textureWidth = width;
    textureHeight = height;

    viewer.resize(
      width: width,
      height: height,
      scaleFactor: (dpr * 100).round(),
      physicalSize: ((width * dpr).round(), (height * dpr).round()),
    );
  }
}
