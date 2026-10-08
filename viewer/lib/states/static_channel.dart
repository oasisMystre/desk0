import 'dart:ui' show VoidCallback;
import 'dart:async' show Timer, StreamSubscription;

typedef Listener<T> = void Function(T);

abstract class StaticChannel<T> {
  static const maxQueue = 64;

  int listenerId = 0;
  StreamSubscription? subscription;

  Timer? pollTimer;

  final Map<int, Map<int, Listener<T>>> subscriptions = {};
  final Map<int, List<T>> queues = {};

  StaticChannel() {
    pollTimer = Timer.periodic(
      const Duration(milliseconds: 400),
      (_) => poll(),
    );
  }

  void dispose() {
    pollTimer?.cancel();
    subscriptions.clear();
    subscription?.cancel();
  }

  Future<void> poll();
  int getIdFromEvent(T event);

  void onEvent(T event) {
    final id = getIdFromEvent(event);
    final listeners = subscriptions[id];

    if (listeners == null || listeners.isEmpty) {
      final queue = queues.putIfAbsent(id, () => []);

      if (queue.length >= maxQueue) queue.removeAt(0);
      queue.add(event);
      return;
    }

    for (final listener in listeners.values) {
      listener(event);
    }
  }

  VoidCallback addEventListener(int id, Listener<T> listener) {
    final listenerId = this.listenerId++;
    subscriptions.putIfAbsent(id, () => {})[listenerId] = listener;
    final queue = queues.remove(id);
    if (queue != null) {
      for (final event in queue) {
        listener(event);
      }
    }

    return () {
      final listeners = subscriptions[id];
      listeners?.remove(listenerId);
      if (listeners != null && listeners.isEmpty) {
        listeners.remove(id);
      }
    };
  }
}
