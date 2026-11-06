import 'package:flutter_test/flutter_test.dart';
import 'package:desk0/main.dart';

void main() {
  testWidgets('MainApplication smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(const MainApplication());
  });
}
