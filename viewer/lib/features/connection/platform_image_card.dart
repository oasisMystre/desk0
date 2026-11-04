import 'package:flutter/widgets.dart';
import 'package:flutter_svg/flutter_svg.dart' show SvgPicture;

class PlatformImageCard extends StatelessWidget {
  final String? os;
  final BorderRadiusGeometry? borderRadius;

  const PlatformImageCard({super.key, required this.os, this.borderRadius});

  SvgPicture get platformIcon {
    switch (os) {
      case "windows":
        return SvgPicture.asset("assets/icons/win.svg");
      case "macos":
        return SvgPicture.asset("assets/icons/mac.svg");
      case "linux":
        return SvgPicture.asset("assets/icons/linux.svg");
      default:
        return SvgPicture.asset("assets/icons/screen.svg");
    }
  }

  Color get platformColor {
    switch (os) {
      case "windows":
        return const Color(0xFF0078D4);
      case "macos":
        return const Color(0xFFA2AAAD);
      case "linux":
        return const Color(0xFFFCC624);
      default:
        return const Color(0xFF6B7280);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: BoxConstraints.expand(),
      decoration: BoxDecoration(
        color: platformColor,
        borderRadius: borderRadius,
      ),
      child: platformIcon,
    );
  }
}
