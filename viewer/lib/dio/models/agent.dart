import 'package:desk0/dio/models/job.dart';

import 'agent_asset.dart';

class Agent {
  Map<int, List<AgentAsset>>? assets;
  final String id, organization, version, createdAt;

  Agent({
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
        "version": String version,
        "created_at": String createdAt,
        "organization": String organization,
        "assets": Map<String, dynamic>? assets,
      } =>
        Agent(
          id: id,
          version: version,
          createdAt: createdAt,
          organization: organization,
          assets: assets?.map(
            (key, assets) => MapEntry(
              int.parse(key),
              (assets as List<dynamic>)
                  .map(
                    (asset) =>
                        AgentAsset.fromJson(asset as Map<String, dynamic>),
                  )
                  .toList(),
            ),
          ),
        ),
      _ => throw const FormatException("Failed to deserialize agent."),
    };
  }
}

class BuildAgent {
  final String job;
  final Agent agent;

  const BuildAgent({required this.job, required this.agent});

  factory BuildAgent.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {'job': String job, 'agent': dynamic agent} => BuildAgent(
        job: job,
        agent: Agent.fromJson(agent as Map<String, dynamic>),
      ),
      _ => throw const FormatException("Failed to deserialize build agent."),
    };
  }
}

class BuildAgentStatus {
  final String? error;
  final JobState state;
  final List<AgentAsset>? assets;

  const BuildAgentStatus({
    required this.state,
    required this.error,
    required this.assets,
  });

  factory BuildAgentStatus.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'state': String state,
        'error': String? error,
        'assets': dynamic assets,
      } =>
        BuildAgentStatus(
          assets: (assets as List<dynamic>?)
              ?.map(
                (asset) => AgentAsset.fromJson(asset as Map<String, dynamic>),
              )
              .toList(),
          error: error,
          state: JobState.fromJson(state),
        ),
      _ => throw const FormatException(
        "Failed to deserialize build agent status.",
      ),
    };
  }
}
