import 'api_impl.dart' show ApiImpl;
import 'models/agent.dart' show Agent, BuildAgent, BuildAgentStatus;

class AgentApi extends ApiImpl {
  AgentApi({required super.client});

  Future<BuildAgent> createAgent(List<String> os) async {
    final response = await client.post("agents", data: {"os": os});

    final result = response.data;
    return BuildAgent.fromJson(result as Map<String, dynamic>);
  }

  Future<BuildAgentStatus> getAgentStatus(String job, String agent) async {
    final response = await client.post(
      "agents/status",
      data: {"job": job, "agent": agent},
    );

    final result = response.data;
    return BuildAgentStatus.fromJson(result as Map<String, dynamic>);
  }

  Future<List<Agent>> getAgents() async {
    final response = await client.get("agents");

    final result = response.data as List<dynamic>;
    return result
        .map((element) => Agent.fromJson(result as Map<String, dynamic>))
        .toList();
  }
}
