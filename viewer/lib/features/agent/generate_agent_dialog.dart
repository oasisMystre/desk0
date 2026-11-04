import 'package:get/get.dart';
import 'package:flutter/cupertino.dart';

import '../../dio/index.dart' show Os;
import '../../states/index.dart' show AgentState;

class GenerateAgentDialog extends StatelessWidget {
  final AgentState agentState = Get.find<AgentState>();

  GenerateAgentDialog({super.key});

  void onGenerateAgent(List<Os> os) async {
    final response = await agentState.createAgent(
      os.map((os) => os.value).toList(),
    );
  }

  @override
  Widget build(BuildContext context) {
    // TODO: implement build
    throw UnimplementedError();
  }
}
