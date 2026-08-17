import 'dart:async';

import 'package:autodev_studio/protocol/models.dart';
import 'package:autodev_studio/protocol/ports.dart';
import 'package:autodev_studio/state/studio_controller.dart';
import 'package:flutter_test/flutter_test.dart';

const ObjectiveSummary objective = ObjectiveSummary(
  schemaVersion: '1',
  id: 'obj-0001',
  repository: 'owner/repo',
  description: 'Implement health endpoint',
  branch: 'autodev/objective-obj0001',
  status: 'queued',
);

ObjectiveEvent event(int index) => ObjectiveEvent(
  schemaVersion: '1',
  eventId: 'evt-$index',
  type: 'objective_running',
  timestamp: DateTime.utc(2026, 8, 17, 12, 0, index % 60),
  objectiveId: objective.id,
  runId: 'run-1',
  taskId: 'task-1',
  data: <String, Object?>{'status': 'running', 'index': index},
);

final class FakeRepository implements ObjectiveRepository {
  List<ObjectiveSummary> values = <ObjectiveSummary>[objective];
  Object? listError;
  bool closed = false;

  @override
  Future<List<ObjectiveSummary>> listObjectives() async {
    if (listError case final Object error) {
      throw error;
    }
    return List<ObjectiveSummary>.of(values);
  }

  @override
  Future<ObjectiveSummary> createObjective({
    required String repository,
    required String description,
    String? branch,
  }) async => objective;

  @override
  Future<void> close() async {
    closed = true;
  }
}

final class FakeEventSource implements ObjectiveEventSource {
  final StreamController<ObjectiveEvent> controller =
      StreamController<ObjectiveEvent>.broadcast();
  bool closed = false;

  @override
  Stream<ObjectiveEvent> connect(Uri endpoint) => controller.stream;

  @override
  Future<void> close() async {
    closed = true;
    await controller.close();
  }
}

void main() {
  test('refreshes objectives and selects an objective', () async {
    final repository = FakeRepository();
    final events = FakeEventSource();
    final controller = StudioController(
      repository: repository,
      eventSource: events,
    );
    addTearDown(controller.shutdown);

    await controller.refreshObjectives();
    expect(controller.objectives, [objective]);

    controller.selectObjective(objective.id);
    expect(controller.selectedObjectiveId, objective.id);
  });

  test('connect appends events and evicts beyond live retention', () async {
    final repository = FakeRepository();
    final source = FakeEventSource();
    final controller = StudioController(
      repository: repository,
      eventSource: source,
      liveEventLimit: 2000,
    );
    addTearDown(controller.shutdown);

    await controller.connect(Uri.parse('http://127.0.0.1:8080/events'));
    for (var index = 0; index < 2001; index += 1) {
      source.controller.add(event(index));
    }
    await pumpEventQueue();

    expect(controller.connectionState, StudioConnectionState.connected);
    expect(controller.events, hasLength(2000));
    expect(controller.events.first.eventId, 'evt-1');
    expect(controller.events.last.eventId, 'evt-2000');
  });

  test('stream error is recoverable state instead of controller failure', () async {
    final repository = FakeRepository();
    final source = FakeEventSource();
    final controller = StudioController(
      repository: repository,
      eventSource: source,
    );
    addTearDown(controller.shutdown);

    await controller.connect(Uri.parse('http://127.0.0.1:8080/events'));
    source.controller.addError(StateError('network lost'));
    await pumpEventQueue();

    expect(controller.connectionState, StudioConnectionState.disconnected);
    expect(controller.lastError, contains('network lost'));
  });

  test('refresh error preserves recoverable error state', () async {
    final repository = FakeRepository()..listError = StateError('server unavailable');
    final source = FakeEventSource();
    final controller = StudioController(
      repository: repository,
      eventSource: source,
    );
    addTearDown(controller.shutdown);

    await controller.refreshObjectives();

    expect(controller.objectives, isEmpty);
    expect(controller.lastError, contains('server unavailable'));
  });

  test('shutdown closes stream and repository resources', () async {
    final repository = FakeRepository();
    final source = FakeEventSource();
    final controller = StudioController(
      repository: repository,
      eventSource: source,
    );

    await controller.connect(Uri.parse('http://127.0.0.1:8080/events'));
    await controller.shutdown();

    expect(repository.closed, isTrue);
    expect(source.closed, isTrue);
    expect(controller.connectionState, StudioConnectionState.disconnected);
  });
}
