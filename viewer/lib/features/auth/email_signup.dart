import 'package:get/get.dart';
import 'package:flutter/material.dart';

import '../../states/auth_state.dart' show AuthState;

class EmailSignup extends StatefulWidget {
  const EmailSignup({super.key});

  @override
  EmailSignupState createState() {
    return EmailSignupState();
  }
}

class EmailSignupState extends State<EmailSignup> {
  final AuthState authState = Get.find<AuthState>();
  final TextEditingController controller = TextEditingController();

  void onLogin() async {
    final email = controller.text;
    if (email.isEmpty) return;

    await authState.loginWithEmail(email);
    controller.clear();
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          TextField(
            controller: controller,
            decoration: const InputDecoration(
              labelText: "Email",
              hintText: "Enter your email",
              border: OutlineInputBorder(),
            ),
          ),
          ElevatedButton(onPressed: onLogin, child: const Text("Sign In")),
        ],
      ),
    );
  }
}
