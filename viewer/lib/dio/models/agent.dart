import 'agent_asset.dart';

class Agent {
  final List<AgentAsset>? assets;
  final String id, organization, version, createdAt;

  const Agent({
    required this.id,
    required this.assets,
    required this.version,
    required this.createdAt,
    required this.organization,
  });

  factory Agent.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        "id": String id,
        "assets": dynamic assets,
        "version": String version,
        "createdAt": String createdAt,
        "organization": String organization,
      } =>
        Agent(
          id: id,
          version: version,
          createdAt: createdAt,
          organization: organization,
          assets: assets != null
              ? (assets as List<dynamic>)
                    .map(
                      (asset) =>
                          AgentAsset.fromJson(asset as Map<String, dynamic>),
                    )
                    .toList()
              : null,
        ),
      _ => throw const FormatException("Failed to deserialize agent."),
    };
  }
}
