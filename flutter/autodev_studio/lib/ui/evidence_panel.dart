import 'package:flutter/material.dart';

import '../protocol/models.dart';

final class EvidencePanel extends StatelessWidget {
  const EvidencePanel({
    required this.objective,
    required this.events,
    super.key,
  });

  final ObjectiveSummary objective;
  final List<ObjectiveEvent> events;

  @override
  Widget build(BuildContext context) {
    final objectiveEvents = events
        .where((event) => event.objectiveId == objective.id)
        .toList(growable: false);
    final blocked = objectiveEvents
        .where((event) => event.data['status'] == 'blocked')
        .length;
    final failed = objectiveEvents
        .where((event) => event.data['status'] == 'failed')
        .length;

    return ListView(
      padding: const EdgeInsets.all(16),
      children: <Widget>[
        Text('Evidence & context', style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: 12),
        Text(objective.repository),
        Text(objective.branch),
        const Divider(height: 24),
        Text('Events: ${objectiveEvents.length}'),
        Text('Blocked: $blocked'),
        Text('Failed: $failed'),
      ],
    );
  }
}
