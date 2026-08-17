import 'dart:async';

import 'package:flutter/foundation.dart';

import '../protocol/models.dart';
import '../protocol/ports.dart';

enum StudioConnectionState { disconnected, connecting, connected }

final class StudioController extends ChangeNotifier {
  StudioController({
    required ObjectiveRepository repository,
    required ObjectiveEventSource eventSource,
    int liveEventLimit = 2000,
  }) : _repository = repository,
       _eventSource = eventSource,
       _liveEventLimit = liveEventLimit {
    if (liveEventLimit <= 0 || liveEventLimit > 10000) {
      throw ArgumentError.value(
        liveEventLimit,
        'liveEventLimit',
        'must be between 1 and 10000',
      );
    }
  }

  final ObjectiveRepository _repository;
  final ObjectiveEventSource _eventSource;
  final int _liveEventLimit;

  List<ObjectiveSummary> _objectives = const <ObjectiveSummary>[];
  List<ObjectiveEvent> _events = const <ObjectiveEvent>[];
  String? _selectedObjectiveId;
  String? _lastError;
  StudioConnectionState _connectionState = StudioConnectionState.disconnected;
  StreamSubscription<ObjectiveEvent>? _subscription;
  bool _shutdown = false;

  List<ObjectiveSummary> get objectives => _objectives;
  List<ObjectiveEvent> get events => _events;
  String? get selectedObjectiveId => _selectedObjectiveId;
  String? get lastError => _lastError;
  StudioConnectionState get connectionState => _connectionState;

  Future<void> refreshObjectives() async {
    _ensureActive();
    try {
      final values = await _repository.listObjectives();
      _objectives = List<ObjectiveSummary>.unmodifiable(values);
      _lastError = null;
    } on Object catch (error) {
      _lastError = error.toString();
    }
    notifyListeners();
  }

  void selectObjective(String? objectiveId) {
    _ensureActive();
    _selectedObjectiveId = objectiveId;
    notifyListeners();
  }

  Future<void> connect(Uri endpoint) async {
    _ensureActive();
    await _subscription?.cancel();
    _connectionState = StudioConnectionState.connecting;
    _lastError = null;
    notifyListeners();

    try {
      final stream = _eventSource.connect(endpoint);
      _subscription = stream.listen(
        _onEvent,
        onError: _onStreamError,
        onDone: _onStreamDone,
      );
      _connectionState = StudioConnectionState.connected;
      notifyListeners();
    } on Object catch (error) {
      _connectionState = StudioConnectionState.disconnected;
      _lastError = error.toString();
      notifyListeners();
    }
  }

  Future<void> disconnect() async {
    if (_shutdown) {
      return;
    }
    await _subscription?.cancel();
    _subscription = null;
    await _eventSource.close();
    _connectionState = StudioConnectionState.disconnected;
    notifyListeners();
  }

  Future<void> shutdown() async {
    if (_shutdown) {
      return;
    }
    _shutdown = true;
    await _subscription?.cancel();
    _subscription = null;
    await _eventSource.close();
    await _repository.close();
    _connectionState = StudioConnectionState.disconnected;
  }

  void _onEvent(ObjectiveEvent event) {
    final next = <ObjectiveEvent>[..._events, event];
    if (next.length > _liveEventLimit) {
      next.removeRange(0, next.length - _liveEventLimit);
    }
    _events = List<ObjectiveEvent>.unmodifiable(next);
    _lastError = null;
    notifyListeners();
  }

  void _onStreamError(Object error, StackTrace stackTrace) {
    _lastError = error.toString();
    _connectionState = StudioConnectionState.disconnected;
    notifyListeners();
  }

  void _onStreamDone() {
    if (_shutdown) {
      return;
    }
    _connectionState = StudioConnectionState.disconnected;
    notifyListeners();
  }

  void _ensureActive() {
    if (_shutdown) {
      throw StateError('StudioController is shut down');
    }
  }

  @override
  void dispose() {
    unawaited(shutdown());
    super.dispose();
  }
}
