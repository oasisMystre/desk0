import 'package:get/get.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import '../../../ffi/flutter.dart' show ViewerHandle;
import '../../../dio/models/index.dart' show Device, DeviceApplication;
import '../../../states/index.dart'
    show
        ApplicationState,
        AppInstallerState,
        IoStaticChannelState,
        TerminalStaticChannelState;
import '../../../states/app_installer_state.dart'
    show
        InstallRunning,
        InstallOutput,
        InstallFailed,
        InstallCompleted,
        InstallDownloadStarted,
        InstallDownloadProgress;

class InstallApplicationOnDeviceDialog extends StatefulWidget {
  final Device device;
  final ViewerHandle viewer;

  const InstallApplicationOnDeviceDialog({
    super.key,
    required this.device,
    required this.viewer,
  });

  @override
  InstallApplicationOnDeviceDialogState createState() {
    return InstallApplicationOnDeviceDialogState();
  }
}

class InstallApplicationOnDeviceDialogState
    extends State<InstallApplicationOnDeviceDialog> {
  late final TerminalStaticChannelState terminalStaticChannelState =
      TerminalStaticChannelState(widget.viewer);
  late final AppInstallerState appInstallerState = AppInstallerState(
    viewer: widget.viewer,
    io: IoStaticChannelState(
      viewer: widget.viewer,
      terminal: terminalStaticChannelState,
    ),
    terminal: terminalStaticChannelState,
  );
  final ApplicationState applicationState = Get.find<ApplicationState>();

  @override
  Widget build(BuildContext context) {
    return Obx(() {
      final applications = applicationState.applications
          .where((application) => application.os == widget.device.os)
          .toList();
      final deviceApplications = <DeviceApplication>[];

      return ListView.builder(
        itemCount: applications.length,
        itemBuilder: (context, index) {
          final application = applications[index];
          final installed =
              deviceApplications.indexWhere(
                (value) => value.application == application.id,
              ) !=
              -1;

          return ListTile(
            leading: application.image == null
                ? null
                : Image.network(application.image!.publicUrl),
            title: Text(application.name),
            trailing: installed
                ? PopupMenuButton(
                    itemBuilder: (context) {
                      return [];
                    },
                  )
                : FilledButton.icon(
                    icon: Icon(CupertinoIcons.download_circle),
                    label: Text("Install"),
                    onPressed: () {
                      final stream = appInstallerState.install(
                        application.asset.publicUrl,
                        "",
                        (path) => "",
                      );

                      stream.listen((event) {
                        switch (event) {
                          case InstallDownloadStarted():
                            // TODO: Handle this case.
                            throw UnimplementedError();
                          case InstallDownloadProgress():
                            // TODO: Handle this case.
                            throw UnimplementedError();
                          case InstallRunning():
                            // TODO: Handle this case.
                            throw UnimplementedError();
                          case InstallOutput():
                            // TODO: Handle this case.
                            throw UnimplementedError();
                          case InstallCompleted():
                            // TODO: Handle this case.
                            throw UnimplementedError();
                          case InstallFailed():
                            // TODO: Handle this case.
                            throw UnimplementedError();
                        }
                      });
                    },
                  ),
          );
        },
      );
    });
  }
}
