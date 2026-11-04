enum JobState {
  failed('failed'),
  active('active'),
  waiting('waiting'),
  delayed('delayed'),
  unknown('unknown'),
  completed('completed'),
  prioritized('prioritized'),
  waitingChildren('waiting-children');

  final String value;

  const JobState(this.value);

  factory JobState.fromJson(String value) {
    return JobState.values.firstWhere(
      (state) => state.value == value,
      orElse: () => JobState.unknown,
    );
  }

  String toJson() => value;
}
