import 'package:flutter/material.dart';

import '../protocol/models.dart';

final class ObjectiveList extends StatelessWidget {
  const ObjectiveList({
    required this.objectives,
    required this.selectedObjectiveId,
    required this.onSelected,
    super.key,
  });

  final List<ObjectiveSummary> objectives;
  final String? selectedObjectiveId;
  final ValueChanged<String> onSelected;

  @override
  Widget build(BuildContext context) {
    if (objectives.isEmpty) {
      return const Center(child: Text('No objectives yet.'));
    }
    return ListView.builder(
      itemCount: objectives.length,
      itemBuilder: (context, index) {
        final objective = objectives[index];
        final selected = objective.id == selectedObjectiveId;
        return Semantics(
          selected: selected,
          label: 'Objective ${objective.description}, ${objective.status}',
          child: ListTile(
            selected: selected,
            title: Text(objective.description),
            subtitle: Text('${objective.repository} • ${objective.status}'),
            onTap: () => onSelected(objective.id),
          ),
        );
      },
    );
  }
}
