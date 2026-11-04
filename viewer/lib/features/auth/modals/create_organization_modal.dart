import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../../states/organization_state.dart';

class CreateOrganizationModal extends StatefulWidget {
  const CreateOrganizationModal({super.key});

  @override
  CreateOrganizationModalState createState() {
    return CreateOrganizationModalState();
  }
}

class CreateOrganizationModalState extends State<CreateOrganizationModal> {
  final TextEditingController controller = TextEditingController();
  final OrganizationState organizationState = Get.find<OrganizationState>();

  void onSubmit() async {
    final name = controller.text;
    if (name.isEmpty) return;

    await organizationState.createOrganization(name);
    controller.clear();
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        TextField(
          controller: controller,
          decoration: const InputDecoration(labelText: "Organization name"),
        ),
        ElevatedButton(
          onPressed: onSubmit,
          child: const Text("Create Organization"),
        ),
      ],
    );
  }
}
