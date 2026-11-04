import 'dart:io';

import 'package:flutter/cupertino.dart';
import 'package:dio/dio.dart' show MultipartFile;
import 'package:file_picker/file_picker.dart' show FilePicker;
import 'package:desktop_drop/desktop_drop.dart' show DropTarget;

import '../../../dio/models/index.dart' show Os;

class ExecutableAssetField extends StatefulWidget {
  final Os os;
  final FormFieldSetter<MultipartFile> onSaved;

  const ExecutableAssetField({
    super.key,
    required this.os,
    required this.onSaved,
  });

  @override
  ExecutableAssetFieldState createState() {
    return ExecutableAssetFieldState();
  }
}

class ExecutableAssetFieldState extends State<ExecutableAssetField> {
  bool loading = false;
  bool dragging = false;

  List<String> get allowedExtensions => switch (widget.os) {
    Os.macos => ['dmg'],
    Os.windows => ['exe', 'msi'],
    Os.linux => ['deb', 'rpm', 'AppImage'],
  };

  bool isAllowed(String name) {
    final extension = name.split('.').last.toLowerCase();
    return allowedExtensions.any(
      (allowed) => allowed.toLowerCase() == extension,
    );
  }

  Future<void> setField(
    FormFieldState<MultipartFile> field,
    String path,
    String name,
  ) async {
    final multipartFile = await MultipartFile.fromFile(path, filename: name);
    field.didChange(multipartFile);
  }

  void onPickFile(FormFieldState<MultipartFile> field) async {
    final file = await FilePicker.pickFile(
      allowedExtensions: allowedExtensions,
    );

    if (file != null) {
      final path = file.uri.toFilePath(windows: Platform.isWindows);
      await setField(field, path, file.name);
    }
  }

  @override
  Widget build(BuildContext context) {
    return FormField<MultipartFile>(
      initialValue: null,
      onSaved: widget.onSaved,
      builder: (field) {
        return DropTarget(
          onDragEntered: (event) => setState(() => dragging = true),
          onDragExited: (event) => setState(() => dragging = false),
          onDragDone: (details) async {
            setState(() => dragging = false);
            for (final file in details.files) {
              if (isAllowed(file.name)) {
                await setField(field, file.path, file.name);
                return;
              }
            }

            field.didChange(null);
          },
          child: GestureDetector(
            onTap: loading ? null : () => onPickFile(field),
            child: Container(
              decoration: BoxDecoration(),
              child: Column(children: []),
            ),
          ),
        );
      },
    );
  }
}
