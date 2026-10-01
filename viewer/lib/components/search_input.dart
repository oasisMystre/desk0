import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart' show SvgPicture;
import 'package:liquid_glass_widgets/liquid_glass_widgets.dart'
    show GlassContainer, GlassButton, LiquidRoundedSuperellipse;

class SearchInput extends StatefulWidget {
  final bool expanded;

  const SearchInput({super.key, this.expanded = false});

  @override
  State<SearchInput> createState() => SearchInputState();
}

class SearchInputState extends State<SearchInput> {
  late bool expanded;

  final controller = TextEditingController();
  final focusNode = FocusNode();

  @override
  void initState() {
    super.initState();

    expanded = widget.expanded;
    focusNode.addListener(_onFocusChanged);
  }

  void _onFocusChanged() {
    if (!focusNode.hasFocus && expanded) {
      setState(() {
        expanded = false;
      });
    }
  }

  void expand() {
    setState(() {
      expanded = true;
    });

    WidgetsBinding.instance.addPostFrameCallback((_) {
      focusNode.requestFocus();
    });
  }

  @override
  void dispose() {
    focusNode.removeListener(_onFocusChanged);
    focusNode.dispose();
    controller.dispose();
    super.dispose();
  }

  Widget buildChild() {
    return Row(
      children: [
        if (expanded)
          Padding(
            padding: EdgeInsets.only(top: 8, bottom: 8, left: 8),
            child: SvgPicture.asset(
              'assets/icons/search.svg',
              colorFilter: ColorFilter.mode(Colors.white, BlendMode.srcIn),
            ),
          )
        else
          GlassButton(
            width: 32,
            height: 32,
            icon: SvgPicture.asset(
              'assets/icons/search.svg',
              colorFilter: ColorFilter.mode(Colors.white, BlendMode.srcIn),
            ),
            onTap: () {
              expand();
            },
          ),

        if (expanded)
          Expanded(
            child: CupertinoTextField(
              controller: controller,
              focusNode: focusNode,
              placeholder: 'Search',
              decoration: null,
            ),
          ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 250),
      curve: Curves.easeInOut,
      width: expanded ? double.infinity : 48,
      constraints: const BoxConstraints(minWidth: 48, maxWidth: 180),
      child: expanded
          ? GlassContainer(
              shape: LiquidRoundedSuperellipse(borderRadius: 100),
              child: buildChild(),
            )
          : buildChild(),
    );
  }
}
