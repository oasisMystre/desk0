import 'api_impl.dart';
import 'models/agent.dart';

class AgentApi extends ApiImpl {
  AgentApi({required super.client});

  Future<Agent> createAgent(List<String> os) async {
    final response = await client.post("agents", data: {"os": os});

    final result = response.data;
    return Agent.fromJson(result as Map<String, dynamic>);
  }

  Future<List<Agent>> getAgents() async {
    final response = await client.get("agents");

    final result = response.data as List<dynamic>;
    return result
        .map((element) => Agent.fromJson(result as Map<String, dynamic>))
        .toList();
  }
}
