import 'dart:io' show Platform;
import 'dart:typed_data' show Uint8List;
import 'dart:ui'
    show PixelFormat, ImageDescriptor, ImmutableBuffer, ImageByteFormat;

import 'package:get/get.dart' show GetxController, Rx, RxT;
import 'package:flutter_custom_cursor/cursor_manager.dart'
    show CursorManager, CursorData;
import 'package:flutter_custom_cursor/flutter_custom_cursor.dart'
    show FlutterCustomMemoryImageCursor;
import 'package:flutter/material.dart'
    show View, MouseCursor, SystemMouseCursors, BuildContext, BoxConstraints;

import '../ffi/flutter.dart' show ViewerHandle;

class DisplayState extends GetxController {
  late Rx<int> width, height;
  final Rx<MouseCursor> cursor = Rx<MouseCursor>(SystemMouseCursors.basic);

  DisplayState({required int width, required int height}) {
    this.width = width.obs;
    this.height = height.obs;
  }

  void resize(int width, int height) {
    if (this.width.value != width) {
      this.width.value = width;
    }

    if (this.height.value != height) {
      this.height.value = height;
    }
  }

  Future<void> onCursorBitmap({
    required int width,
    required int height,
    required Uint8List bitmapData,
    required int hotspotX,
    required int hotspotY,
  }) async {
    final cursorName = await CursorManager.instance.registerCursor(
      CursorData()
        ..name = "desk0"
        ..buffer = (Platform.isWindows
            ? rgbaToBgra(width, height, bitmapData)
            : await rgbaToPng(width, height, bitmapData))
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

    final width = DisplayState.alignDown(constraints.maxWidth.round(), 64);
    final height = DisplayState.alignDown(constraints.maxHeight.round(), 2);

    if (width <= 0 || height <= 0) {
      return;
    }

    if (this.width.value == width && this.height.value == height) {
      return;
    }

    this.width.value = width;
    this.height.value = height;

    viewer.resize(
      width: width,
      height: height,
      scaleFactor: (dpr * 100).round(),
      physicalSize: ((width * dpr).round(), (height * dpr).round()),
    );
  }

  Future<Uint8List> rgbaToPng(int width, int height, Uint8List rgba) async {
    final buffer = await ImmutableBuffer.fromUint8List(rgba);
    final descriptor = ImageDescriptor.raw(
      buffer,
      width: width,
      height: height,
      pixelFormat: PixelFormat.rgba8888,
    );

    final codec = await descriptor.instantiateCodec();
    final frame = await codec.getNextFrame();
    final png = await frame.image.toByteData(format: ImageByteFormat.png);

    buffer.dispose();
    descriptor.dispose();
    codec.dispose();

    return png!.buffer.asUint8List(png.offsetInBytes, png.lengthInBytes);
  }

  Uint8List rgbaToBgra(int width, int height, Uint8List rgba) {
    final destination = Uint8List(rgba.length);

    for (int index = 0; index < rgba.length; index += 4) {
      destination[index] = rgba[index + 2];
      destination[index + 1] = rgba[index + 1];
      destination[index + 2] = rgba[index];
      destination[index + 3] = rgba[index + 3];
    }

    return destination;
  }

  static int alignDown(int v, int a) => (v ~/ a) * a;
  static int alignUp(int v, int a) => ((v + a - 1) ~/ a) * a;
}
