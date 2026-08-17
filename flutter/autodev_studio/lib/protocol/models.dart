const String publicSchemaVersion = '1';

final class ObjectiveSummary {
  const ObjectiveSummary({
    required this.schemaVersion,
    required this.id,
    required this.repository,
    required this.description,
    required this.branch,
    required this.status,
  });

  final String schemaVersion;
  final String id;
  final String repository;
  final String description;
  final String branch;
  final String status;

  factory ObjectiveSummary.fromJson(Map<String, Object?> json) {
    return ObjectiveSummary(
      schemaVersion: _schemaVersion(json),
      id: _requiredString(json, 'id'),
      repository: _requiredString(json, 'repository'),
      description: _requiredString(json, 'description'),
      branch: _requiredString(json, 'branch'),
      status: _requiredString(json, 'status'),
    );
  }
}

final class ObjectiveEvent {
  const ObjectiveEvent({
    required this.schemaVersion,
    required this.eventId,
    required this.type,
    required this.timestamp,
    required this.objectiveId,
    required this.runId,
    required this.taskId,
    required this.data,
  });

  final String schemaVersion;
  final String eventId;
  final String type;
  final DateTime timestamp;
  final String objectiveId;
  final String? runId;
  final String? taskId;
  final Map<String, Object?> data;

  factory ObjectiveEvent.fromJson(Map<String, Object?> json) {
    return ObjectiveEvent(
      schemaVersion: _schemaVersion(json),
      eventId: _requiredString(json, 'event_id'),
      type: _requiredString(json, 'type'),
      timestamp: _requiredDateTime(json, 'timestamp'),
      objectiveId: _requiredString(json, 'objective_id'),
      runId: _nullableString(json, 'run_id'),
      taskId: _nullableString(json, 'task_id'),
      data: Map<String, Object?>.unmodifiable(_requiredMap(json, 'data')),
    );
  }
}

final class EvidenceSummary {
  const EvidenceSummary({
    required this.schemaVersion,
    required this.evidenceId,
    required this.objectiveId,
    required this.runId,
    required this.taskId,
    required this.kind,
    required this.status,
    required this.observedAt,
    required this.detail,
  });

  final String schemaVersion;
  final String evidenceId;
  final String objectiveId;
  final String? runId;
  final String? taskId;
  final String kind;
  final String status;
  final DateTime observedAt;
  final String detail;

  factory EvidenceSummary.fromJson(Map<String, Object?> json) {
    return EvidenceSummary(
      schemaVersion: _schemaVersion(json),
      evidenceId: _requiredString(json, 'evidence_id'),
      objectiveId: _requiredString(json, 'objective_id'),
      runId: _nullableString(json, 'run_id'),
      taskId: _nullableString(json, 'task_id'),
      kind: _requiredString(json, 'kind'),
      status: _requiredString(json, 'status'),
      observedAt: _requiredDateTime(json, 'observed_at'),
      detail: _requiredString(json, 'detail', allowEmpty: true),
    );
  }
}

final class SourceSpan {
  const SourceSpan({required this.start, required this.end});

  final int start;
  final int end;

  factory SourceSpan.fromJson(Map<String, Object?> json) {
    final start = _requiredInt(json, 'start');
    final end = _requiredInt(json, 'end');
    if (start < 0 || end < start) {
      throw const FormatException('span must satisfy 0 <= start <= end');
    }
    return SourceSpan(start: start, end: end);
  }
}

final class GraphNode {
  const GraphNode({
    required this.id,
    required this.label,
    required this.kind,
    required this.file,
    required this.enclosing,
    required this.span,
  });

  final String id;
  final String label;
  final String kind;
  final String file;
  final String? enclosing;
  final SourceSpan span;

  factory GraphNode.fromJson(Map<String, Object?> json) {
    return GraphNode(
      id: _requiredString(json, 'id'),
      label: _requiredString(json, 'label'),
      kind: _requiredString(json, 'kind'),
      file: _requiredString(json, 'file'),
      enclosing: _nullableString(json, 'enclosing'),
      span: SourceSpan.fromJson(_requiredMap(json, 'span')),
    );
  }
}

final class GraphEdge {
  const GraphEdge({
    required this.id,
    required this.from,
    required this.to,
    required this.kind,
  });

  final String id;
  final String from;
  final String to;
  final String kind;

  factory GraphEdge.fromJson(Map<String, Object?> json) {
    return GraphEdge(
      id: _requiredString(json, 'id'),
      from: _requiredString(json, 'from'),
      to: _requiredString(json, 'to'),
      kind: _requiredString(json, 'kind'),
    );
  }
}

final class CodeGraphSnapshot {
  const CodeGraphSnapshot({
    required this.schemaVersion,
    required this.snapshotId,
    required this.repository,
    required this.revision,
    required this.nodes,
    required this.edges,
  });

  final String schemaVersion;
  final String snapshotId;
  final String repository;
  final String? revision;
  final List<GraphNode> nodes;
  final List<GraphEdge> edges;

  factory CodeGraphSnapshot.fromJson(Map<String, Object?> json) {
    final nodes = _requiredList(json, 'nodes')
        .map((value) => GraphNode.fromJson(_objectMap(value, 'nodes item')))
        .toList(growable: false);
    final edges = _requiredList(json, 'edges')
        .map((value) => GraphEdge.fromJson(_objectMap(value, 'edges item')))
        .toList(growable: false);
    return CodeGraphSnapshot(
      schemaVersion: _schemaVersion(json),
      snapshotId: _requiredString(json, 'snapshot_id'),
      repository: _requiredString(json, 'repository'),
      revision: _nullableString(json, 'revision'),
      nodes: List<GraphNode>.unmodifiable(nodes),
      edges: List<GraphEdge>.unmodifiable(edges),
    );
  }
}

final class ConnectivityStatus {
  const ConnectivityStatus({
    required this.schemaVersion,
    required this.sourceId,
    required this.kind,
    required this.state,
    required this.protocol,
    required this.latencyMs,
    required this.observedAt,
    required this.detail,
  });

  final String schemaVersion;
  final String sourceId;
  final String kind;
  final String state;
  final String protocol;
  final int? latencyMs;
  final DateTime observedAt;
  final String detail;

  factory ConnectivityStatus.fromJson(Map<String, Object?> json) {
    final latencyMs = _nullableInt(json, 'latency_ms');
    if (latencyMs != null && latencyMs < 0) {
      throw const FormatException('latency_ms must be non-negative');
    }
    return ConnectivityStatus(
      schemaVersion: _schemaVersion(json),
      sourceId: _requiredString(json, 'source_id'),
      kind: _requiredString(json, 'kind'),
      state: _requiredString(json, 'state'),
      protocol: _requiredString(json, 'protocol'),
      latencyMs: latencyMs,
      observedAt: _requiredDateTime(json, 'observed_at'),
      detail: _requiredString(json, 'detail', allowEmpty: true),
    );
  }
}

String _schemaVersion(Map<String, Object?> json) {
  final value = _requiredString(json, 'schema_version');
  if (value != publicSchemaVersion) {
    throw FormatException('unsupported schema_version: $value');
  }
  return value;
}

String _requiredString(
  Map<String, Object?> json,
  String key, {
  bool allowEmpty = false,
}) {
  final value = json[key];
  if (value is! String || (!allowEmpty && value.isEmpty)) {
    throw FormatException('$key must be ${allowEmpty ? 'a string' : 'a non-empty string'}');
  }
  return value;
}

String? _nullableString(Map<String, Object?> json, String key) {
  final value = json[key];
  if (value == null) {
    return null;
  }
  if (value is! String) {
    throw FormatException('$key must be a string or null');
  }
  return value;
}

int _requiredInt(Map<String, Object?> json, String key) {
  final value = json[key];
  if (value is! int) {
    throw FormatException('$key must be an integer');
  }
  return value;
}

int? _nullableInt(Map<String, Object?> json, String key) {
  final value = json[key];
  if (value == null) {
    return null;
  }
  if (value is! int) {
    throw FormatException('$key must be an integer or null');
  }
  return value;
}

DateTime _requiredDateTime(Map<String, Object?> json, String key) {
  final raw = _requiredString(json, key);
  final parsed = DateTime.tryParse(raw);
  if (parsed == null) {
    throw FormatException('$key must be an ISO-8601 timestamp');
  }
  return parsed;
}

Map<String, Object?> _requiredMap(Map<String, Object?> json, String key) {
  return _objectMap(json[key], key);
}

Map<String, Object?> _objectMap(Object? value, String key) {
  if (value is! Map) {
    throw FormatException('$key must be an object');
  }
  try {
    return Map<String, Object?>.from(value);
  } on TypeError {
    throw FormatException('$key must have string keys');
  }
}

List<Object?> _requiredList(Map<String, Object?> json, String key) {
  final value = json[key];
  if (value is! List) {
    throw FormatException('$key must be an array');
  }
  return List<Object?>.from(value);
}
