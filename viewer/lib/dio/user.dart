import 'apl_impl.dart';
import 'models/user.dart';
import 'models/organization.dart';

class CreateUser {
  final User user;
  final Organization organization;

  const CreateUser({required this.user, required this.organization});

  factory CreateUser.fromJson(Map<String, dynamic> json) {
    return switch (json) {
      {
        'user': Map<String, dynamic> user,
        'organization': Map<String, dynamic> organization,
      } =>
        CreateUser(
          user: User.fromJson(user),
          organization: Organization.fromJson(organization),
        ),
      _ => throw const FormatException("failed to load user"),
    };
  }
}

class UserApi extends ApiImpl {
  const UserApi({required super.client});

  // Future<User> getUser() async {
  //   final response = await http.get(buildPath('user'));

  //   if (response.statusCode == 200) {
  //     final result = response.data as Map<String, dynamic>;
  //     return User.fromJson(result);
  //   } else {
  //     throw Exception(response.body);
  //   }
  // }

  Future<CreateUser> createUser() async {
    final response = await client.post('/users');

    if (response.statusCode == 200) {
      final result = response.data as Map<String, dynamic>;
      return CreateUser.fromJson(result);
    } else {
      throw Exception(response.data);
    }
  }

  Future<User> deleteUser() async {
    final response = await client.delete('/user');

    if (response.statusCode == 200) {
      final result = response.data as Map<String, dynamic>;
      return User.fromJson(result);
    } else {
      throw Exception(response.data);
    }
  }
}
