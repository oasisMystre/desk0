import 'package:flutter/material.dart';
import 'package:dio/dio.dart' show MultipartFile;
import 'package:get/get.dart' hide MultipartFile;

import '../../states/application_state.dart' show ApplicationState;
import '../../dio/index.dart' show Api, Script, CreateApplication, Os;
import './fields/index.dart'
    show ImageField, OsField, ArgsField, ExecutableAssetField;

class CreateAppDialog extends StatefulWidget {
  const CreateAppDialog({super.key});

  @override
  CreateAppDialogState createState() {
    return CreateAppDialogState();
  }
}

class CreateAppDialogState extends State<CreateAppDialog> {
  int currentFormIndex = 0;

  final Api api = Get.find<Api>();
  final ApplicationState applicationState = Get.find<ApplicationState>();

  late Os os;
  late String name;
  late MultipartFile asset;

  MultipartFile? image;
  List<String>? runArgs, installArgs, uninstallArgs;

  List<String> get stepTitles => [
    'Select Os',
    'Metadata',
    'Upload Executable',
    'Add Scripts',
  ];

  List<Widget> get stepScreens => [
    OsStep(onSaved: (value) => os = value),
    MetadataStep(
      onNameSaved: (value) => name = value,
      onImageSaved: (value) => image = value,
    ),
    ExecuteableAssetStep(os: os, onSaved: (value) => asset = value),
    ScriptStep(
      onRunArgsSaved: (value) => runArgs = value,
      onInstallArgsSaved: (value) => installArgs = value,
      onUninstallArgsSaved: (value) => uninstallArgs = value,
    ),
  ];

  void onInstall() async {
    await applicationState.createApplication(
      CreateApplication(
        os: os.value,
        asset: asset,
        image: image,
        name: name,
        scripts: Script.fromJson({
          'run': {'args': runArgs},
          'install': {'args': installArgs},
          'uninstall': {'args': uninstallArgs},
        }),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: MediaQuery.sizeOf(context).width * 0.5,
      height: MediaQuery.sizeOf(context).height * 0.7,

      child: Row(
        children: [
          SizedBox(
            width: (MediaQuery.sizeOf(context).width * 0.5) * 0.3,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: List.generate(stepTitles.length, (index) {
                final value = stepTitles[index];
                final active = currentFormIndex == index;

                return ListTile(
                  selected: active,
                  title: Text(value),
                  onTap: () {
                    if (index < currentFormIndex) {
                      setState(() => currentFormIndex = index);
                    }
                  },
                );
              }),
            ),
          ),
          Expanded(
            child: IndexedStack(index: currentFormIndex, children: stepScreens),
          ),
        ],
      ),
    );
  }
}

class OsStep extends StatelessWidget {
  final ValueChanged<Os> onSaved;

  const OsStep({super.key, required this.onSaved});

  @override
  Widget build(BuildContext context) {
    return OsField(
      value: null,
      onSaved: (value) {
        if (value != null) onSaved(value);
      },
    );
  }
}

class MetadataStep extends StatelessWidget {
  final ValueChanged<String> onNameSaved;
  final ValueChanged<MultipartFile?> onImageSaved;

  const MetadataStep({
    super.key,
    required this.onNameSaved,
    required this.onImageSaved,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        TextFormField(
          onSaved: (value) {
            if (value != null) {
              onNameSaved(value);
            }
          },
        ),
        ImageField(onSaved: onImageSaved),
      ],
    );
  }
}

class ExecuteableAssetStep extends StatelessWidget {
  final Os os;
  final ValueChanged<MultipartFile> onSaved;

  const ExecuteableAssetStep({
    super.key,
    required this.os,
    required this.onSaved,
  });

  @override
  Widget build(BuildContext context) {
    return ExecutableAssetField(
      os: os,
      onSaved: (value) {
        if (value != null) onSaved(value);
      },
    );
  }
}

class ScriptStep extends StatelessWidget {
  final ValueChanged<List<String>?> onRunArgsSaved;
  final ValueChanged<List<String>?> onInstallArgsSaved;
  final ValueChanged<List<String>?> onUninstallArgsSaved;

  const ScriptStep({
    super.key,
    required this.onRunArgsSaved,
    required this.onInstallArgsSaved,
    required this.onUninstallArgsSaved,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        ArgsField(onSaved: onRunArgsSaved),
        ArgsField(onSaved: onInstallArgsSaved),
        ArgsField(onSaved: onUninstallArgsSaved),
      ],
    );
  }
}
