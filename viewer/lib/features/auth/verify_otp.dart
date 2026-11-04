import 'package:get/get.dart';
import 'package:flutter/cupertino.dart';

import '../../states/auth_state.dart' show AuthState;

class VerifyOtp extends StatefulWidget {
  const VerifyOtp({super.key});

  @override
  VerifyOtpState createState() {
    return VerifyOtpState();
  }
}

class VerifyOtpState extends State<VerifyOtp> {
  final AuthState authState = Get.find<AuthState>();
  final TextEditingController controller = TextEditingController();

  void onVerify() async {
    final String otp = controller.text;
    if (otp.isEmpty) return;

    await authState.verifyOTP(otp);
    controller.clear();
  }

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(children: []);
  }
}
