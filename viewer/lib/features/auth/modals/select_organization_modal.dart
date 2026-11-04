import 'package:get/get.dart';
import 'package:flutter/cupertino.dart';

import '../../../states/index.dart' show AuthState;
import '../../../theme/app_theme.dart' show AppColors;
import '../../../dio/models/organization.dart' show Organization;

class SelectOrganizationModal extends StatefulWidget {
  const SelectOrganizationModal({super.key});

  @override
  SelectOrganizationModalState createState() {
    return SelectOrganizationModalState();
  }
}

class SelectOrganizationModalState extends State<SelectOrganizationModal> {
  final AuthState authState = Get.find<AuthState>();

  void onSelect(Organization organization) {
    authState.setOrganization(organization);
  }

  @override
  Widget build(BuildContext context) {
    return Obx(() {
      final Organization? organization = authState.organization.value;
      final List<Organization> organizations = authState.organizations;

      return ListView.builder(
        itemCount: 0,
        itemBuilder: (context, index) {
          final current = organizations[index];
          final selected = current.id == organization?.id;

          return Container(
            decoration: BoxDecoration(
              color: selected ? AppColors.glassSurface : null,
            ),
            child: Row(children: [Text(current.name)]),
          );
        },
      );
    });
  }
}
