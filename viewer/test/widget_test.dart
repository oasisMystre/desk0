import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:desk0/components/index.dart';
import 'package:desk0/screens/home.dart';

void main() {
  testWidgets('SideNav and Toolbar smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: ThemeData.dark(),
        home: const Scaffold(
          body: Row(
            children: [
              SideNav(selectedRoute: '/'),
              Expanded(
                child: Column(
                  children: [
                    Toolbar(),
                    Expanded(child: HomeScreen()),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );

    expect(find.text('Favorites'), findsOneWidget);
    expect(find.text('Devices'), findsOneWidget);
    expect(find.text('Apps'), findsOneWidget);
    expect(find.text('Search'), findsOneWidget);
    expect(find.text('64.224.17.188'), findsOneWidget);
    expect(find.text('Administrator'), findsOneWidget);
    expect(find.text('PC Connection'), findsOneWidget);
  });
}
