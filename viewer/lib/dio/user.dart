import 'api_impl.dart' show ApiImpl;
import 'models/user.dart' show User;

class UserApi extends ApiImpl {
  const UserApi({required super.client});

  Future<User> getUser() async {
    final response = await client.get('user');

    final result = response.data as Map<String, dynamic>;
    return User.fromJson(result);
  }


  Future<User> deleteUser() async {
    final response = await client.delete('/user');

    final result = response.data as Map<String, dynamic>;
    return User.fromJson(result);
  }
}
