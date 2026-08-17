import 'dart:async';

import 'package:autodev_studio/protocol/models.dart';
import 'package:autodev_studio/protocol/ports.dart';
import 'package:autodev_studio/state/studio_controller.dart';
import 'package:autodev_studio/ui/studio_screen.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

const ObjectiveSummary objective = ObjectiveSummary(
  schemaVersion: '1',
  id: 'obj-0001',
  repository: 'owner/repo',
  description: 'Implement health endpoint',
  branch: 'autodev/objective-obj0001',
  status: 'running',
);

ObjectiveEvent statusEvent(String type, String status, String reason) => ObjectiveEvent(
  schemaVersion: '1',
  eventId: 'evt-$status',
  type: type,
  timestamp: DateTime.utc(2026, 8, 17, 12),
  objectiveId: objective.id,
  runId: 'run-1',
  taskId: 'task-1',
  data: <String, Object?>{'status': status, 'reason': reason},
);

final class FakeRepository implements ObjectiveRepository {
  @override
  Future<List<ObjectiveSummary>> listObjectives() async => <ObjectiveSummary>[objective];

  @override
  Future<ObjectiveSummary> createObjective({
    required String repository,
    required String description,
    String? branch,
  }) async => objective;

  @override
  Future<void> close() async {}
}

final class FakeEventSource implements ObjectiveEventSource {
  final StreamController<ObjectiveEvent> events = StreamController<ObjectiveEvent>.broadcast();

  @override
  Stream<ObjectiveEvent> connect(Uri endpoint) => events.stream;

  @override
  Future<void> close() async {
    await events.close();
  }
}

Future<StudioController> controllerWithData() async {
  final source = FakeEventSource();
  final controller = StudioController(
    repository: FakeRepository(),
    eventSource: source,
  );
  await controller.refreshObjectives();
  await controller.connect(Uri.parse('http://127.0.0.1:8080/events'));
  source.events
    ..add(statusEvent('objective_blocked', 'blocked', 'trusted approval required'))
    ..add(statusEvent('objective_failed', 'failed', 'verification failed'));
  await Future<void>.delayed(Duration.zero);
  return controller;
}

Future<void> setSurface(WidgetTester tester, double width) async {
  tester.view.physicalSize = Size(width, 800);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
}

void main() {
  testWidgets('wide layout renders objectives timeline and evidence panes', (tester) async {
    await setSurface(tester, 1200);
    final controller = await controllerWithData();
    addTearDown(controller.shutdown);
    controller.selectObjective(objective.id);

    await tester.pumpWidget(
      MaterialApp(home: StudioScreen(controller: controller)),
    );

    expect(find.byKey(const Key('objectives-pane')), findsOneWidget);
    expect(find.byKey(const Key('timeline-pane')), findsOneWidget);
    expect(find.byKey(const Key('evidence-pane')), findsOneWidget);
    expect(find.text('trusted approval required'), findsOneWidget);
    expect(find.text('verification failed'), findsOneWidget);
    expect(find.bySemanticsLabel('Objective blocked: trusted approval required'), findsOneWidget);
    expect(find.bySemanticsLabel('Objective failed: verification failed'), findsOneWidget);
  });

  testWidgets('compact desktop layout uses two panes', (tester) async {
    await setSurface(tester, 900);
    final controller = await controllerWithData();
    addTearDown(controller.shutdown);
    controller.selectObjective(objective.id);

    await tester.pumpWidget(
      MaterialApp(home: StudioScreen(controller: controller)),
    );

    expect(find.byKey(const Key('objectives-pane')), findsOneWidget);
    expect(find.byKey(const Key('timeline-pane')), findsOneWidget);
    expect(find.byKey(const Key('evidence-pane')), findsNothing);
  });

  testWidgets('narrow layout starts with one objective pane', (tester) async {
    await setSurface(tester, 600);
    final controller = await controllerWithData();
    addTearDown(controller.shutdown);

    await tester.pumpWidget(
      MaterialApp(home: StudioScreen(controller: controller)),
    );

    expect(find.byKey(const Key('objectives-pane')), findsOneWidget);
    expect(find.byKey(const Key('timeline-pane')), findsNothing);
    expect(find.byKey(const Key('evidence-pane')), findsNothing);
  });

  testWidgets('narrow selection transitions to timeline and exposes back navigation', (tester) async {
    await setSurface(tester, 600);
    final controller = await controllerWithData();
    addTearDown(controller.shutdown);
    controller.selectObjective(objective.id);

    await tester.pumpWidget(
      MaterialApp(home: StudioScreen(controller: controller)),
    );

    expect(find.byKey(const Key('objectives-pane')), findsNothing);
    expect(find.byKey(const Key('timeline-pane')), findsOneWidget);
    expect(find.byTooltip('Back to objectives'), findsOneWidget);
  });
}
