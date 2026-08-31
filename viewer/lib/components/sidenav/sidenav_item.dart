import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../theme/app_theme.dart';

class SideNavData {
  final String name;
  final Widget Function(Color color) icon;
  final String path;

  SideNavData({required this.icon, required this.name, required this.path});
}

class SideNavItem extends StatelessWidget {
  final bool selected;
  final bool focused;

  final SideNavData data;
  final VoidCallback? onPressed;

  const SideNavItem({
    super.key,
    required this.data,
    required this.focused,
    required this.selected,
    this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onPressed,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(8),
            color: selected
                ? AppColors.sidebarSelected
                : CupertinoColors.transparent,
          ),
          child: Padding(
            padding: const EdgeInsets.all(8),
            child: Row(
              spacing: 8,
              children: [
                data.icon(selected ? AppColors.accentBlue : Colors.white54),
                Text(
                  data.name,
                  style: TextTheme.of(context).labelMedium?.copyWith(
                    color: selected
                        ? AppColors.accentBlue
                        : AppColors.textPrimary,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class SideNavGroup extends StatefulWidget {
  final int selectedIndex;
  final List<SideNavData> group;
  final void Function(int, SideNavData)? onSelected;

  const SideNavGroup({
    super.key,
    required this.group,
    required this.selectedIndex,
    this.onSelected,
  });

  @override
  State<SideNavGroup> createState() {
    return SideNavState();
  }
}

class SideNavState extends State<SideNavGroup> {
  late int selectedIndex;
  final FocusNode focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    selectedIndex = widget.selectedIndex;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) focusNode.requestFocus();
    });
  }

  @override
  void didUpdateWidget(covariant SideNavGroup oldWidget) {
    super.didUpdateWidget(oldWidget);

    if (oldWidget.selectedIndex != widget.selectedIndex) {
      selectedIndex = widget.selectedIndex;
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        focusNode.requestFocus();
      }
    });
  }

  @override
  void dispose() {
    focusNode.dispose();
    super.dispose();
  }

  void select(int index) {
    if (widget.group.isEmpty) return;

    final length = widget.group.length;
    index = (index % length + length) % length;

    setState(() {
      selectedIndex = index;
    });

    widget.onSelected?.call(index, widget.group[index]);

    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        focusNode.requestFocus();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    if (widget.group.isEmpty) return const SizedBox.shrink();

    return Focus(
      focusNode: focusNode,
      autofocus: true,
      canRequestFocus: true,
      descendantsAreFocusable: false,
      onKeyEvent: (node, event) {
        if (event is KeyDownEvent) {
          if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
            select(selectedIndex + 1);
            return KeyEventResult.handled;
          } else if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
            select(selectedIndex - 1);
            return KeyEventResult.handled;
          }
        }
        return KeyEventResult.ignored;
      },
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () => focusNode.requestFocus(),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (int index = 0; index < widget.group.length; index++)
              SideNavItem(
                data: widget.group[index],
                selected: index == selectedIndex,
                focused: focusNode.hasFocus && index == selectedIndex,
                onPressed: () {
                  select(index);
                  focusNode.requestFocus();
                },
              ),
          ],
        ),
      ),
    );
  }
}
