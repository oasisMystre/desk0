import 'package:flutter/material.dart';

import '../theme/app_theme.dart' show AppColors;

class TabButton extends StatefulWidget {
  final String data;
  final bool selected;
  final VoidCallback onClose;
  final VoidCallback onPressed;

  const TabButton({
    super.key,
    required this.selected,
    required this.data,
    required this.onClose,
    required this.onPressed,
  });

  @override
  TabButtonState createState() {
    return TabButtonState();
  }
}

class TabButtonState extends State<TabButton> {
  bool hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (event) {
        setState(() {
          hovered = true;
        });
      },
      onExit: (event) {
        setState(() {
          hovered = false;
        });
      },
      child: GestureDetector(
        onTap: widget.onPressed,
        child: Container(
          padding: EdgeInsets.only(left: 8, right: 4),
          decoration: BoxDecoration(
            color: widget.selected
                ? Colors.black26
                : hovered
                ? Colors.black12
                : Colors.transparent,
            border: Border(
              bottom: BorderSide(
                width: 1,
                color: widget.selected
                    ? AppColors.accentBlue
                    : Colors.transparent,
              ),
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                widget.data,
                style: TextTheme.of(context).labelMedium?.copyWith(
                  color: widget.selected
                      ? AppColors.accentBlue
                      : hovered
                      ? AppColors.textPrimary
                      : AppColors.textSecondary,
                ),
              ),
              SizedBox(
                width: 24,
                child: AnimatedOpacity(
                  opacity: hovered ? 1 : 0,
                  duration: const Duration(milliseconds: 100),
                  child: IconButton(
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(
                      minWidth: 24,
                      minHeight: 24,
                    ),
                    iconSize: 14,
                    color: widget.selected ? null : AppColors.textPrimary,
                    icon: const Icon(Icons.close),
                    onPressed: hovered ? widget.onClose : null,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
