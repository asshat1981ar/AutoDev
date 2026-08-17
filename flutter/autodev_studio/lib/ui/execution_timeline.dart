import 'package:flutter/material.dart';

import '../protocol/models.dart';

final class ExecutionTimeline extends StatelessWidget {
  const ExecutionTimeline({
    required this.events,
    required this.objectiveId,
    super.key,
  });

  final List<ObjectiveEvent> events;
  final String objectiveId;

  @override
  Widget build(BuildContext context) {
    final visible = events
        .where((event) => event.objectiveId == objectiveId)
        .toList(growable: false);
    if (visible.isEmpty) {
      return const Center(child: Text('No execution events yet.'));
    }
    return ListView.builder(
      itemCount: visible.length,
      itemBuilder: (context, index) {
        final event = visible[index];
        final status = event.data['status'] is String
            ? event.data['status']! as String
            : event.type;
        final reason = event.data['reason'] is String
            ? event.data['reason']! as String
            : null;
        final semanticLabel = reason == null
            ? 'Objective $status'
            : 'Objective $status: $reason';
        return Semantics(
          label: semanticLabel,
          child: ListTile(
            leading: Icon(_iconFor(status)),
            title: Text(_titleFor(event.type)),
            subtitle: reason == null ? null : Text(reason),
          ),
        );
      },
    );
  }

  static IconData _iconFor(String status) {
    return switch (status) {
      'blocked' => Icons.lock_outline,
      'failed' => Icons.error_outline,
      'verifying' => Icons.fact_check_outlined,
      'completed' => Icons.check_circle_outline,
      _ => Icons.pending_outlined,
    };
  }

  static String _titleFor(String type) {
    return type
        .replaceAll('objective_', '')
        .replaceAll('_', ' ')
        .split(' ')
        .where((part) => part.isNotEmpty)
        .map((part) => '${part[0].toUpperCase()}${part.substring(1)}')
        .join(' ');
  }
}
