import 'dart:ui';

import 'package:flutter/material.dart' show CircularProgressIndicator, Colors;
import 'package:flutter/cupertino.dart'
    show
        StatelessWidget,
        Widget,
        BuildContext,
        Color,
        EdgeInsets,
        SizedBox,
        TextStyle,
        BorderRadius,
        BoxDecoration,
        MainAxisSize,
        Text,
        TextAlign,
        Column,
        Container,
        Center;

import '../../../l10n/generated/app_localizations.dart' show AppLocalizations;

enum ConnectionPhase { connecting, connected, reconnecting, failed }

class ConnectionOverlay extends StatelessWidget {
  final ConnectionPhase phase;
  final String? message;

  const ConnectionOverlay({super.key, required this.phase, this.message});

  @override
  Widget build(BuildContext context) {
    final localization = AppLocalizations.of(context)!;

    final title = switch (phase) {
      ConnectionPhase.connecting => localization.connecting,
      ConnectionPhase.reconnecting => localization.reconnecting,
      ConnectionPhase.failed => localization.connection_failed,
      _ => throw UnimplementedError(),
    };

    return Container(
      color: const Color(0xCC111318),
      child: Center(
        child: Container(
          padding: const EdgeInsets.all(24),
          decoration: BoxDecoration(
            color: const Color(0xEE20232A),
            borderRadius: BorderRadius.circular(12),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (phase != ConnectionPhase.failed)
                const CircularProgressIndicator(),
              if (phase != ConnectionPhase.failed) const SizedBox(height: 16),
              Text(
                title,
                style: const TextStyle(
                  color: Colors.white,
                  fontStyle: FontStyle.normal,
                  fontWeight: FontWeight.normal,
                  fontSize: 12,
                ),
              ),
              if (message != null) ...[
                const SizedBox(height: 8),
                Text(
                  message!,
                  textAlign: TextAlign.center,
                  style: const TextStyle(color: Colors.white70),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
