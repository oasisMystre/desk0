import 'package:flutter/cupertino.dart';

class ConnectionItem {
  final String id;
  final String ipAddress;
  final String username;
  final String type;
  final bool isFavorite;

  const ConnectionItem({
    required this.id,
    required this.ipAddress,
    required this.username,
    this.type = 'PC Connection',
    this.isFavorite = true,
  });
}

class ConnectionCard extends StatefulWidget {
  final ConnectionItem connection;
  final VoidCallback? onTap;

  const ConnectionCard({
    super.key,
    required this.connection,
    this.onTap,
  });

  @override
  State<ConnectionCard> createState() => _ConnectionCardState();
}

class _ConnectionCardState extends State<ConnectionCard> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          width: 320,
          height: 200,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(16),
            border: Border.all(
              color: _isHovered
                  ? const Color(0xFF0A84FF).withValues(alpha: 0.6)
                  : const Color(0xFF323642),
              width: _isHovered ? 1.5 : 1.0,
            ),
            boxShadow: [
              BoxShadow(
                color: CupertinoColors.black.withValues(alpha: _isHovered ? 0.45 : 0.25),
                blurRadius: _isHovered ? 16 : 10,
                offset: const Offset(0, 4),
              ),
            ],
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(15),
            child: Stack(
              children: [
                // Custom macOS Monterey Graphic Background
                Positioned.fill(
                  child: CustomPaint(
                    painter: _MontereyWallpaperPainter(),
                  ),
                ),

                // Dark vignette overlay
                Positioned.fill(
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      gradient: LinearGradient(
                        begin: Alignment.topCenter,
                        end: Alignment.bottomCenter,
                        colors: [
                          CupertinoColors.black.withValues(alpha: 0.15),
                          CupertinoColors.black.withValues(alpha: 0.45),
                        ],
                      ),
                    ),
                  ),
                ),

                // Card Contents
                Padding(
                  padding: const EdgeInsets.all(14.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      // Top Chip: "PC Connection"
                      Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 10,
                          vertical: 4,
                        ),
                        decoration: BoxDecoration(
                          color: const Color(0xE514161A),
                          borderRadius: BorderRadius.circular(6),
                          border: Border.all(
                            color: CupertinoColors.white.withValues(alpha: 0.08),
                            width: 0.8,
                          ),
                        ),
                        child: Text(
                          widget.connection.type,
                          style: const TextStyle(
                            color: CupertinoColors.white,
                            fontSize: 11,
                            fontWeight: FontWeight.w600,
                            letterSpacing: -0.1,
                          ),
                        ),
                      ),

                      // Bottom Info: IP & User
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            widget.connection.ipAddress,
                            style: const TextStyle(
                              color: CupertinoColors.white,
                              fontSize: 18,
                              fontWeight: FontWeight.w700,
                              letterSpacing: -0.3,
                            ),
                          ),
                          const SizedBox(height: 2),
                          Text(
                            widget.connection.username,
                            style: const TextStyle(
                              color: Color(0xFFD1D5DB),
                              fontSize: 13,
                              fontWeight: FontWeight.w400,
                            ),
                          ),
                        ],
                      ),
                    ],
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

class _MontereyWallpaperPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    // Base gradient fill
    final Rect rect = Offset.zero & size;
    final Paint bgPaint = Paint()
      ..shader = const LinearGradient(
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
        colors: [
          Color(0xFF1A1C24),
          Color(0xFF2C1635),
          Color(0xFF5A1846),
          Color(0xFF1F102B),
        ],
        stops: [0.0, 0.4, 0.7, 1.0],
      ).createShader(rect);
    canvas.drawRect(rect, bgPaint);

    // Deep Magenta / Purple Wave in upper-right
    final Path wave1 = Path();
    wave1.moveTo(size.width * 0.35, 0);
    wave1.cubicTo(
      size.width * 0.45,
      size.height * 0.35,
      size.width * 0.6,
      size.height * 0.15,
      size.width,
      size.height * 0.2,
    );
    wave1.lineTo(size.width, 0);
    wave1.close();

    final Paint wave1Paint = Paint()
      ..shader = const LinearGradient(
        begin: Alignment.topCenter,
        end: Alignment.bottomRight,
        colors: [
          Color(0xFF7B1FA2),
          Color(0xFFAD1457),
        ],
      ).createShader(rect);
    canvas.drawPath(wave1, wave1Paint);

    // S-curve Magenta-Purple Glow Ribbons
    final Path wave2 = Path();
    wave2.moveTo(0, size.height * 0.45);
    wave2.cubicTo(
      size.width * 0.35,
      size.height * 0.55,
      size.width * 0.4,
      size.height * 0.05,
      size.width,
      size.height * 0.35,
    );
    wave2.lineTo(size.width, size.height);
    wave2.lineTo(0, size.height);
    wave2.close();

    final Paint wave2Paint = Paint()
      ..shader = const LinearGradient(
        begin: Alignment.topRight,
        end: Alignment.bottomLeft,
        colors: [
          Color(0xFF6A1B9A),
          Color(0xFF38006B),
          Color(0xFF160D26),
        ],
      ).createShader(rect);
    canvas.drawPath(wave2, wave2Paint);

    // Dynamic Flowing Ribbon (Silver / Violet light highlight)
    final Path ribbon = Path();
    ribbon.moveTo(0, size.height * 0.38);
    ribbon.cubicTo(
      size.width * 0.3,
      size.height * 0.48,
      size.width * 0.45,
      size.height * 0.08,
      size.width * 0.78,
      size.height * 0.28,
    );
    ribbon.cubicTo(
      size.width * 0.9,
      size.height * 0.38,
      size.width * 0.95,
      size.height * 0.42,
      size.width,
      size.height * 0.45,
    );
    ribbon.lineTo(size.width, size.height * 0.5);
    ribbon.cubicTo(
      size.width * 0.85,
      size.height * 0.45,
      size.width * 0.7,
      size.height * 0.32,
      size.width * 0.48,
      size.height * 0.2,
    );
    ribbon.cubicTo(
      size.width * 0.3,
      size.height * 0.58,
      size.width * 0.1,
      size.height * 0.48,
      0,
      size.height * 0.44,
    );
    ribbon.close();

    final Paint ribbonPaint = Paint()
      ..shader = const LinearGradient(
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
        colors: [
          Color(0xFF8B9CB8),
          Color(0xFFB39DDB),
          Color(0xFFBA68C8),
          Color(0xFF673AB7),
        ],
      ).createShader(rect);
    canvas.drawPath(ribbon, ribbonPaint);

    // Front soft dark wave
    final Path frontWave = Path();
    frontWave.moveTo(0, size.height * 0.5);
    frontWave.cubicTo(
      size.width * 0.25,
      size.height * 0.65,
      size.width * 0.4,
      size.height * 0.38,
      size.width * 0.65,
      size.height * 0.55,
    );
    frontWave.cubicTo(
      size.width * 0.85,
      size.height * 0.68,
      size.width * 0.95,
      size.height * 0.65,
      size.width,
      size.height * 0.65,
    );
    frontWave.lineTo(size.width, size.height);
    frontWave.lineTo(0, size.height);
    frontWave.close();

    final Paint frontPaint = Paint()
      ..shader = const LinearGradient(
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
        colors: [
          Color(0xFF1C1A2E),
          Color(0xFF131120),
        ],
      ).createShader(rect);
    canvas.drawPath(frontWave, frontPaint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
