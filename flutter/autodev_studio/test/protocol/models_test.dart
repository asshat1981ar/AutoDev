import 'dart:convert';
import 'dart:io';

import 'package:autodev_studio/protocol/models.dart';
import 'package:flutter_test/flutter_test.dart';

Map<String, Object?> fixture(String name) {
  final file = File(
    '${Directory.current.path}/../../protocols/public/v1/fixtures/$name',
  );
  final decoded = jsonDecode(file.readAsStringSync());
  return Map<String, Object?>.from(decoded as Map);
}

void main() {
  test('decodes canonical objective summary', () {
    final model = ObjectiveSummary.fromJson(
      fixture('objective-summary.queued.json'),
    );
    expect(model.id, 'obj-0001');
    expect(model.repository, 'owner/repo');
    expect(model.description, 'Implement health endpoint');
    expect(model.status, 'queued');
  });

  test('decodes canonical objective event and preserves unknown event types', () {
    final json = fixture('objective-event.queued.json');
    final model = ObjectiveEvent.fromJson(json);
    expect(model.schemaVersion, '1');
    expect(model.type, 'objective_queued');
    expect(model.objectiveId, 'obj-0001');
    expect(model.runId, isNull);
    expect(model.data['status'], 'queued');

    final future = Map<String, Object?>.from(json)..['type'] = 'future_event_type';
    expect(ObjectiveEvent.fromJson(future).type, 'future_event_type');
  });

  test('decodes canonical evidence summary', () {
    final model = EvidenceSummary.fromJson(
      fixture('evidence-summary.passed.json'),
    );
    expect(model.kind, 'unit_tests');
    expect(model.status, 'passed');
    expect(model.taskId, 'task-0001');
  });

  test('decodes canonical code graph snapshot', () {
    final model = CodeGraphSnapshot.fromJson(
      fixture('code-graph-snapshot.json'),
    );
    expect(model.snapshotId, 'graph-0001');
    expect(model.nodes, hasLength(1));
    expect(model.nodes.single.label, 'MainActivity');
    expect(model.nodes.single.span.start, 0);
    expect(model.nodes.single.span.end, 120);
    expect(model.edges, isEmpty);
  });

  test('decodes canonical connectivity status', () {
    final model = ConnectivityStatus.fromJson(
      fixture('connectivity-status.ready.json'),
    );
    expect(model.sourceId, 'mcp-filesystem');
    expect(model.kind, 'mcp');
    expect(model.state, 'ready');
    expect(model.latencyMs, 12);
  });

  test('rejects missing and wrongly typed required fields', () {
    expect(
      () => ObjectiveSummary.fromJson(<String, Object?>{
        'schema_version': '1',
        'repository': 'owner/repo',
        'description': 'Missing id',
        'branch': 'main',
        'status': 'queued',
      }),
      throwsFormatException,
    );

    final invalidEvent = fixture('objective-event.queued.json')
      ..['objective_id'] = 42;
    expect(
      () => ObjectiveEvent.fromJson(invalidEvent),
      throwsFormatException,
    );
  });
}
