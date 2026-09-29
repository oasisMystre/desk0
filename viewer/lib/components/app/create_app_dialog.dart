import 'package:flutter/material.dart';

class CreateAppDialog extends StatelessWidget {
  const CreateAppDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: MediaQuery.sizeOf(context).width * 0.5,
      height: MediaQuery.sizeOf(context).height * 0.7,

      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Icon(Icons.upload_file_rounded, size: 56),
            Column(
              children: [
                Text(
                  "Drag and drop your executable file",
                  style: TextTheme.of(context).labelLarge
                      ?.copyWith(color: Colors.white),
                ),
                Text(
                  "Max File size 250 MB",
                  style: TextTheme.of(context).labelMedium
                      ?.copyWith(color: Colors.white70),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
