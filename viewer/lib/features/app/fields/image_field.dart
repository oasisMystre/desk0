import 'package:flutter/cupertino.dart';
import 'package:dio/dio.dart' show MultipartFile;

class ImageField extends StatelessWidget {
  final FormFieldSetter<MultipartFile> onSaved;
  const ImageField({super.key, required this.onSaved});

  @override
  Widget build(BuildContext context) {
    // TODO: implement build
    throw UnimplementedError();
  }
}
