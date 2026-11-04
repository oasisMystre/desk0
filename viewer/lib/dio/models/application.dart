import 'package:dio/dio.dart';

import 'attachment.dart';

class ScriptArgs {
  final List<String> args;

  const ScriptArgs({required this.args});

  Map<String, dynamic> toJson() {
    return {'args': args};
  }

  factory ScriptArgs.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {'args': dynamic args} => ScriptArgs(
        args: (args as List<dynamic>).cast<String>().toList(),
      ),
      _ => throw const FormatException("Failed to deserialize script args."),
    };
  }
}

class Script {
  final ScriptArgs? run;
  final ScriptArgs? install;
  final ScriptArgs? uninstall;

  const Script({
    required this.run,
    required this.install,
    required this.uninstall,
  });

  Map<String, dynamic> toJson() {
    return {
      'run': run?.toJson(),
      'install': install?.toJson(),
      'uninstall': uninstall?.toJson(),
    };
  }

  factory Script.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'run': Map<String, dynamic>? run,
        'install': Map<String, dynamic>? install,
        'uninstall': Map<String, dynamic>? uninstall,
      } =>
        Script(
          run: run == null ? null : ScriptArgs.fromJson(run),
          install: install == null ? null : ScriptArgs.fromJson(install),
          uninstall: uninstall == null ? null : ScriptArgs.fromJson(uninstall),
        ),
      _ => throw const FormatException("Failed to deserialize scripts."),
    };
  }
}

class Application {
  final Script? scripts;
  final String? image;
  final String id, os, name, asset, organization, createdAt;

  Application({
    required this.id,
    required this.os,
    required this.name,
    required this.asset,
    required this.scripts,
    required this.image,
    required this.createdAt,
    required this.organization,
  });

  factory Application.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'os': String os,
        'name': String name,
        'asset': String asset,
        'image': String? image,
        'created_at': String createdAt,
        'organization': String organization,
        'scripts': Map<String, dynamic> scripts,
      } =>
        Application(
          id: id,
          os: os,
          name: name,
          image: image,
          asset: asset,
          createdAt: createdAt,
          organization: organization,
          scripts: Script.fromJson(scripts),
        ),
      _ => throw const FormatException("Failed to deserialize application."),
    };
  }
}

class ExpandedApplication {
  final Script? scripts;
  final Attachment asset;
  final Attachment? image;
  final String id, os, name, organization, createdAt;

  ExpandedApplication({
    required this.id,
    required this.os,
    required this.name,
    required this.asset,
    required this.scripts,
    required this.image,
    required this.createdAt,
    required this.organization,
  });

  factory ExpandedApplication.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'id': String id,
        'os': String os,
        'name': String name,
        'created_at': String createdAt,
        'organization': String organization,
        'asset': Map<String, dynamic> asset,
        'image': Map<String, dynamic>? image,
        'scripts': Map<String, dynamic> scripts,
      } =>
        ExpandedApplication(
          id: id,
          os: os,
          name: name,
          createdAt: createdAt,
          organization: organization,
          asset: Attachment.fromJson(asset),
          scripts: Script.fromJson(scripts),
          image: image == null ? null : Attachment.fromJson(image),
        ),
      _ => throw const FormatException("Failed to deserialize application."),
    };
  }
}

class CreateApplication {
  final String os, name;
  final Script? scripts;
  final MultipartFile asset;
  final MultipartFile? image;

  CreateApplication({
    required this.os,
    required this.name,
    required this.asset,
    required this.image,
    required this.scripts,
  });

  FormData toFormData() {
    return FormData.fromMap({
      'os': os,
      'name': name,
      'asset': asset,
      if (image != null) 'image': image,
      if (scripts != null) 'scripts': scripts?.toJson(),
    });
  }
}
