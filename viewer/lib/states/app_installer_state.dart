import 'dart:convert' show utf8;
import 'dart:io' show Platform;
import 'dart:async' show Stream, StreamController;

import '../ffi/flutter.dart' show ViewerHandle;
import 'io_channel_state.dart' show IoStaticChannelState;
import 'terminal_channel_state.dart' show TerminalStaticChannelState;
import '../ffi/types/static_channels.dart' show SseStaticChannelInputEvent;
import '../ffi/types/static_channels/terminal.dart'
    show
        SseTerminalMode,
        SseTerminalOutputEvent_Data,
        SseTerminalOutputEvent_Error,
        SseTerminalOutputEvent_Exit,
        SseTerminalOutputEvent_Connected;
import '../ffi/types/static_channels/io.dart'
    show
        SseIoInputEvent,
        SseIoOutputEvent_DownloadFailed,
        SseIoOutputEvent_DownloadStarted,
        SseIoOutputEvent_DownloadProgress,
        SseIoOutputEvent_DownloadCompleted;

class AppInstallerState {
  final ViewerHandle viewer;
  final IoStaticChannelState io;
  final TerminalStaticChannelState terminal;

  AppInstallerState({
    required this.io,
    required this.viewer,
    required this.terminal,
  });

  Stream<InstallEvent> install(
    String url,
    String path,
    Function(String) getInstallCommand,
  ) async* {
    final id = io.getUniqueId();

    late final void Function() removeIoListener;
    late final void Function() removeTerminalListener;

    bool closed = false;
    final controller = StreamController<InstallEvent>(
      onCancel: () {
        if (closed) return;
        removeIoListener();
        removeTerminalListener();
      },
    );

    void close() {
      if (closed) return;
      closed = true;

      removeIoListener();
      removeTerminalListener();
      controller.close();
    }

    removeIoListener = io.addEventListener(id, (event) {
      switch (event) {
        case SseIoOutputEvent_DownloadStarted():
          controller.add(const InstallDownloadStarted());
        case SseIoOutputEvent_DownloadProgress(:final downloaded, :final total):
          controller.add(
            InstallDownloadProgress(downloaded.toInt(), total?.toInt()),
          );
        case SseIoOutputEvent_DownloadCompleted(:final path):
          controller.add(const InstallRunning());
          final command = getInstallCommand(
            utf8.decode(path, allowMalformed: true),
          );
          terminal.send(id, command, mode: SseTerminalMode.sync_);
        case SseIoOutputEvent_DownloadFailed(:final description):
          controller.add(
            InstallFailed(utf8.decode(description, allowMalformed: true)),
          );
          close();
      }
    });

    removeTerminalListener = terminal.addEventListener(id, (event) {
      switch (event) {
        case SseTerminalOutputEvent_Data(:final data):
          controller.add(
            InstallOutput(utf8.decode(data, allowMalformed: true)),
          );
        case SseTerminalOutputEvent_Exit(:final code, :final description):
          if (code == 0) {
            controller.add(const InstallCompleted());
          } else {
            controller.add(
              InstallFailed("[Process exited with $code] $description"),
            );
            close();
          }
        case SseTerminalOutputEvent_Error(:final description, :final code):
          controller.add(
            InstallFailed("Terminal exited with $code $description"),
          );
          close();
        case SseTerminalOutputEvent_Connected():
          break;
      }
    });

    try {
      await viewer.sendStaticChannelInputEvents(
        events: [
          SseStaticChannelInputEvent.io(
            SseIoInputEvent.download(
              id: id,
              uri: utf8.encode(url),
              path: utf8.encode(path),
            ),
          ),
        ],
      );
    } catch (error) {
      close();
      rethrow;
    }

    yield* controller.stream;
  }

  static String getDesk0AppsDirectory() {
    if (Platform.isMacOS) {
      return '/Library/Application Support/Desk0/apps';
    }

    if (Platform.isWindows) {
      return r'C:\ProgramData\Desk0\apps';
    }

    if (Platform.isLinux) {
      return '/var/lib/desk0/apps';
    }

    throw UnsupportedError('Unsupported platform');
  }
}

sealed class InstallEvent {
  const InstallEvent();
}

class InstallDownloadStarted extends InstallEvent {
  const InstallDownloadStarted();
}

class InstallDownloadProgress extends InstallEvent {
  final int downloaded;
  final int? total;

  const InstallDownloadProgress(this.downloaded, this.total);
}

class InstallRunning extends InstallEvent {
  const InstallRunning();
}

class InstallOutput extends InstallEvent {
  final String data;

  const InstallOutput(this.data);
}

class InstallCompleted extends InstallEvent {
  const InstallCompleted();
}

class InstallFailed extends InstallEvent {
  final String description;

  const InstallFailed(this.description);
}
