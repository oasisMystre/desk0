import 'attachment.dart' show Attachment;

class AgentAsset {
  final int version;
  final String agent, os, arch;
  final Attachment attachment;

  const AgentAsset({
    required this.agent,
    required this.os,
    required this.arch,
    required this.version,
    required this.attachment,
  });

  factory AgentAsset.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        "os": String os,
        "arch": String arch,
        "agent": String agent,
        "version": num version,
        "attachment": Map<String, dynamic> attachment,
      } =>
        AgentAsset(
          os: os,
          arch: arch,
          agent: agent,
          version: version.toInt(),
          attachment: Attachment.fromJson(attachment),
        ),
      _ => throw const FormatException("Failed to deserialize agent asset."),
    };
  }
}
