import 'dart:io';

import 'package:flutter/services.dart';

class TrafficLight {
  final double x;
  final double y;
  final double width;
  final double height;
  final double buttonDiameter;

  const TrafficLight({
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    required this.buttonDiameter,
  });
}

class Channel {
  static Future<TrafficLight> getTrafficLightFrame() async {
    if (Platform.isMacOS) {
      final channel = MethodChannel('com.desk0.viewer/channels');

      final result = await channel.invokeMethod<Map>('getTrafficLightFrame');
      final data = result!.map(
        (key, value) => MapEntry(key as String, (value as num).toDouble()),
      );

      return TrafficLight(
        x: data['x']!,
        y: data['y']!,
        width: data['width']!,
        height: data['height']!,
        buttonDiameter: data['buttonDiameter']!,
      );
    }

    return TrafficLight(x: 0, y: 0, width: 0, height: 0, buttonDiameter: 0);
  }
}
