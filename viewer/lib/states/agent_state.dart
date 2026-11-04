import 'dart:async';

import 'package:get/get.dart';

import '../dio/index.dart' show Agent, Api, JobState;

class AgentState extends GetxController {
  static int agentBuildTimeoutSeconds = 15;

  final Api api;
  final RxList<Agent> agents = <Agent>[].obs;

  AgentState({required this.api});

  Future<Agent> createAgent(List<String> os) async {
    final response = await api.agent.createAgent(os);
    final agent = response.agent;

    agents.add(agent);

    final stopwatch = Stopwatch()..start();

    try {
      while (stopwatch.elapsed.inSeconds < agentBuildTimeoutSeconds) {
        final status = await api.agent.getAgentStatus(
          response.job,
          response.agent.id,
        );

        if (status.state == JobState.failed) {
          throw Exception(status.error);
        }

        final assets = status.assets;

        if (assets != null && assets.isNotEmpty) {
          if (agent.assets == null) {
            agent.assets = {};
          }

          for (final asset in assets) {
            agent.assets?.putIfAbsent(asset.version, () => []).add(asset);
          }
        }

        if (status.state == JobState.completed) {
          break;
        }

        Future.delayed(Duration(seconds: 1));
      }

      if (stopwatch.elapsed.inSeconds >= agentBuildTimeoutSeconds) {
        throw TimeoutException('Agent build timed out');
      }

      return response.agent;
    } finally {
      stopwatch.stop();

      final index = agents.indexWhere((value) => value.id == agent.id);
      if (index != -1) {
        agents[index] = agent;
      }
    }
  }

  Future<List<Agent>> getAgents() async {
    final List<Agent> responses = await api.agent.getAgents();
    agents.value = responses;

    return responses;
  }
}
