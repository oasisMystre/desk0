import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

class ArgsField extends StatefulWidget {
  final List<String> value;
  final FormFieldSetter<List<String>> onSaved;

  const ArgsField({super.key, required this.onSaved, this.value = const []});

  @override
  ArgFieldState createState() {
    return ArgFieldState();
  }
}

class ArgFieldState extends State<ArgsField> {
  late List<String> args;
  final controller = TextEditingController();

  void addArgs(FormFieldState<List<String>> field) {
    final arg = controller.text.trim();

    if (arg.isEmpty) return;
    final current = field.value == null ? <String>[] : field.value!;
    if (current.contains(arg)) {
      controller.clear();
      return;
    }

    final updated = [...current, arg];
    field.didChange(updated);

    controller.clear();
  }

  void removeArg(FormFieldState<List<String>> field, String arg) {
    final updated = [...(field.value == null ? <String>[] : field.value!)]
      ..remove(arg);
    field.didChange(updated);
  }

  @override
  Widget build(BuildContext context) {
    return FormField<List<String>>(
      onSaved: widget.onSaved,
      initialValue: widget.value,
      builder: (field) {
        final List<String> args = field.value == null ? [] : field.value!;

        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: args.map((arg) {
                return DecoratedBox(
                  decoration: BoxDecoration(
                    color: CupertinoColors.systemGrey5,
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        arg,
                        style: Theme.of(context).textTheme.labelSmall
                            ?.copyWith(fontFamily: 'monospace'),
                      ),
                      GestureDetector(
                        onTap: () => removeArg(field, arg),

                        child: Icon(
                          CupertinoIcons.xmark_circle_fill,
                          color: CupertinoColors.systemGrey,
                        ),
                      ),
                    ],
                  ),
                );
              }).toList(),
            ),
            Expanded(
              child: CupertinoTextField(
                controller: controller,
                autocorrect: false,
                textCapitalization: TextCapitalization.none,
                onSubmitted: (event) => addArgs(field),
              ),
            ),
          ],
        );
      },
    );
  }
}
