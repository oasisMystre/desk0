import 'package:get/get.dart';

import '../../dio/index.dart' show Api, Organization;

class OrganizationState extends GetxController {
  final Api api;

  Rxn<Organization> organization = Rxn<Organization>();
  RxList<Organization> organizations = <Organization>[].obs;

  OrganizationState({required this.api});

  Future<void> createOrganization(String name) async {
    final organization = await api.organization.createOrganization(name);
    organizations.add(organization);
    setOrganization(organization);
  }

  void setOrganization(Organization? value) {
    organization.value = value;
    if (value != null) api.updateOrganizationId(value.id);
  }
}
