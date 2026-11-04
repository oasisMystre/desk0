import 'package:get/get.dart';

import '../dio/index.dart' show Api, ExpandedApplication, CreateApplication;

class ApplicationState extends GetxController {
  final Api api;
  final RxList<ExpandedApplication> applications = <ExpandedApplication>[].obs;

  ApplicationState({required this.api});

  Future<List<ExpandedApplication>> getApplications() async {
    final response = await api.application.getApplications();
    applications.value = response;
    return response;
  }

  Future<void> createApplication(CreateApplication application) async {
    final response = await api.application.createApplication(application);
    applications.add(response);
  }

  Future<void> deleteApplication(String application) async {
    final response = await api.application.deleteApplication(application);
    applications.removeWhere((application) => application.id == response.id);
  }
}
