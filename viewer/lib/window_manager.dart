import 'dart:io';
import 'dart:async';

import 'package:desk0/extensions.dart';
import 'package:get/instance_manager.dart';
import 'package:multiview_desktop/multiview_desktop.dart';
import 'package:flutter/material.dart' hide ConnectionState;

import 'states/connection_state.dart' show ConnectionState;
import 'features/connection/index.dart' show OpenConnection;

abstract class WindowManager {
  int? viewId;
  Future<int>? openingWindowFuture;
  Future<void>? closingWindowFuture;

  FutureOr<int> onOpen();

  void onWindowOpened(int publicId) {
    viewId = publicId;
  }

  void onWindowClosed(int publicId) {
    if (viewId == publicId) {
      viewId = null;
      openingWindowFuture = null;
    }
  }

  Future<int> open({bool focus = false}) async {
    while (closingWindowFuture != null) {
      await closingWindowFuture;
    }

    if (viewId != null) {
      if (focus) await this.focus(id: viewId);
      return viewId!;
    }

    final existing = openingWindowFuture;
    if (existing != null) {
      final id = await existing;
      final closing = closingWindowFuture;
      if (closing != null) await closing;

      if (focus) await this.focus(id: id);
      return id;
    }

    final opening = Future<int>.sync(onOpen);
    openingWindowFuture = opening;

    return await opening
        .then((id) async {
          viewId = id;
          if (focus) await this.focus(id: id);
          return id;
        })
        .whenComplete(() {
          if (identical(openingWindowFuture, opening)) {
            openingWindowFuture = null;
          }
        });
  }

  Future<void> focus({int? id}) async {
    final windowId = id ?? viewId;
    if (windowId == null) return;

    final window = MultiViewDesktop.fromId(windowId);

    await window.restore();
    if (!await window.isVisible()) {
      await window.show();
    }

    await window.focus();
  }

  Future<void> show() async {
    final id = viewId;
    if (id == null) {
      await open(focus: true);
      return;
    }

    final window = MultiViewDesktop.fromId(id);
    await window.restore();
    await window.show();
    await window.focus();
  }

  Future<void> hide() async {
    if (viewId != null) {
      final window = MultiViewDesktop.fromId(viewId!);
      await window.hide();
    }
  }

  Future<void> close({bool quit = false}) async {
    final existing = closingWindowFuture;
    if (existing != null) return existing;

    late final Future<void> closing;

    closing = performClose(quit).whenComplete(() {
      if (identical(closingWindowFuture, closing)) {
        closingWindowFuture = null;
      }
    });

    closingWindowFuture = closing;
    return closing;
  }

  Future<void> performClose(bool quit) async {
    final opening = openingWindowFuture;

    if (opening != null) {
      await opening.ignoreError();
    }

    final id = viewId;
    viewId = null;

    if (quit) {
      await MultiViewDesktop.closeApp().ignoreError();
      exit(0);
    }

    if (id != null) {
      final window = MultiViewDesktop.fromId(id);
      await window.closeWindow();
    }
  }
}

class ConnectionWindowManager extends WindowManager {
  ConnectionState get connectionState => Get.find<ConnectionState>();
  static final ConnectionWindowManager instance = ConnectionWindowManager._();

  ConnectionWindowManager._();

  @override
  Future<int> onOpen() {
    return openWindow(
      (context, publicId) => const OpenConnection(),
      options: const WindowOptions(
        title: "Desk0 Viewer",
        size: Size(1024, 720),
        minimumSize: Size(640, 400),
        hideAppFromTaskbar: false,
        windowButtonVisibility: true,
        titleBarStyle: TitleBarStyle.hidden,
        backgroundColor: Colors.transparent,
      ),
    );
  }
}
