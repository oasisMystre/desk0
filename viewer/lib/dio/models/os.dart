enum Os {
  macos('macos', 'assets/icons/mac.svg'),
  linux('linux', 'assets/icons/linux.svg'),
  windows('windows', 'assets/icons/windows.svg');

  const Os(this.value, this.icon);

  final String value;
  final String icon;

  factory Os.fromJson(String value) {
    return Os.values.firstWhere(
      (os) => os.value == value,
      orElse: () => throw FormatException('Unknown OS: $value'),
    );
  }

  String toJson() => value;
}
