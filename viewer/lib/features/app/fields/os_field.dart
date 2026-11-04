import 'package:flutter/material.dart';
import 'package:flutter_svg/svg.dart' show SvgPicture;

import '../../../dio/models/os.dart';

class OsField extends StatelessWidget {
  final Os? value;
  final FormFieldSetter<Os> onSaved;

  const OsField({super.key, required this.value, required this.onSaved});

  @override
  Widget build(BuildContext context) {
    return FormField(
      initialValue: value,
      onSaved: onSaved,
      builder: (FormFieldState<Os> field) {
        return RadioGroup<Os>(
          groupValue: field.value,
          onChanged: field.didChange,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: Os.values.map((os) {
              return InkWell(
                onTap: () => field.didChange(os),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    SvgPicture.asset(os.icon),
                    Text(os.name),
                    Radio<Os>(value: os),
                  ],
                ),
              );
            }).toList(),
          ),
        );
      },
    );
  }
}
