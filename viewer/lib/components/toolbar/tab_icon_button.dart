import 'package:flutter/material.dart';
import 'package:liquid_glass_widgets/liquid_glass_widgets.dart'
    show
        GlassButton,
        LiquidRoundedSuperellipse,
        LiquidRoundedRectangle,
        GlassContainer;

class TabIconData {
  final String? id;
  final String name;
  final Widget Function(Color? color) icon;

  const TabIconData({required this.name, required this.icon, this.id});
}

class TabIconButton extends StatelessWidget {
  final bool selected;
  final bool showShader;
  final TabIconData data;
  final VoidCallback onPressed;

  const TabIconButton({
    super.key,
    required this.data,
    required this.onPressed,
    this.showShader = true,
    this.selected = false,
  });

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return Tooltip(
      message: data.name,
      child: showShader
          ? GlassButton(
              width: 32,
              height: 32,
              icon: data.icon(Colors.white),
              shape: LiquidRoundedSuperellipse(borderRadius: 100),
              onTap: onPressed,
            )
          : IconButton(
              icon: data.icon(colorScheme.onSurface),
              onPressed: onPressed,
              isSelected: selected,
              hoverColor: colorScheme.onSurface.withValues(alpha: 0.12),
              style: IconButton.styleFrom(
                backgroundColor: selected
                    ? colorScheme.onSurface.withValues(alpha: 0.12)
                    : null,
              ),
            ),
    );
  }
}

class TabIconGroupButton extends StatelessWidget {
  final int? selectedIndex;
  final List<TabIconData> group;
  final Function(int, TabIconData) onSelected;

  const TabIconGroupButton({
    super.key,
    required this.group,
    required this.onSelected,
    this.selectedIndex,
  });

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        Positioned.fill(
          child: GlassContainer(
            shape: LiquidRoundedRectangle(borderRadius: 100),
          ),
        ),
        Container(
          padding: EdgeInsets.all(4),
          child: Row(
            spacing: 4,
            children: group.asMap().entries.map((entry) {
              final index = entry.key;
              final data = entry.value;

              return TabIconButton(
                data: data,
                showShader: false,
                selected: index == selectedIndex,
                onPressed: () {
                  onSelected(index, data);
                },
              );
            }).toList(),
          ),
        ),
      ],
    );
  }
}
