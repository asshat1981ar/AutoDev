import 'package:flutter/material.dart';

import '../protocol/models.dart';
import '../state/studio_controller.dart';
import 'evidence_panel.dart';
import 'execution_timeline.dart';
import 'objective_list.dart';

class StudioScreen extends StatelessWidget {
  const StudioScreen({this.controller, super.key});

  final StudioController? controller;

  @override
  Widget build(BuildContext context) {
    final state = controller;
    if (state == null) {
      return Scaffold(
        appBar: AppBar(title: const Text('AutoDev Studio')),
        body: Center(
          child: Semantics(
            label: 'AutoDev connection status: disconnected',
            child: const Text('Disconnected'),
          ),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('AutoDev Studio'),
        actions: <Widget>[
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Center(child: Text(_connectionLabel(state.connectionState))),
          ),
        ],
      ),
      body: AnimatedBuilder(
        animation: state,
        builder: (context, _) {
          return LayoutBuilder(
            builder: (context, constraints) {
              if (constraints.maxWidth >= 1100) {
                return _wideWorkspace(state);
              }
              if (constraints.maxWidth >= 700) {
                return _compactWorkspace(state);
              }
              return _narrowWorkspace(state);
            },
          );
        },
      ),
    );
  }

  static Widget _wideWorkspace(StudioController controller) {
    final selected = _selectedObjective(controller);
    return Row(
      children: <Widget>[
        SizedBox(
          key: const Key('objectives-pane'),
          width: 300,
          child: _objectives(controller),
        ),
        const VerticalDivider(width: 1),
        Expanded(
          key: const Key('timeline-pane'),
          child: _timeline(controller, selected),
        ),
        const VerticalDivider(width: 1),
        SizedBox(
          key: const Key('evidence-pane'),
          width: 320,
          child: _evidence(controller, selected),
        ),
      ],
    );
  }

  static Widget _compactWorkspace(StudioController controller) {
    final selected = _selectedObjective(controller);
    return Row(
      children: <Widget>[
        SizedBox(
          key: const Key('objectives-pane'),
          width: 300,
          child: _objectives(controller),
        ),
        const VerticalDivider(width: 1),
        Expanded(
          key: const Key('timeline-pane'),
          child: _timeline(controller, selected),
        ),
      ],
    );
  }

  static Widget _narrowWorkspace(StudioController controller) {
    final selected = _selectedObjective(controller);
    if (selected == null) {
      return KeyedSubtree(
        key: const Key('objectives-pane'),
        child: _objectives(controller),
      );
    }
    return Column(
      children: <Widget>[
        Align(
          alignment: Alignment.centerLeft,
          child: IconButton(
            tooltip: 'Back to objectives',
            onPressed: () => controller.selectObjective(null),
            icon: const Icon(Icons.arrow_back),
          ),
        ),
        Expanded(
          key: const Key('timeline-pane'),
          child: _timeline(controller, selected),
        ),
      ],
    );
  }

  static Widget _objectives(StudioController controller) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        const _PaneHeading('Objectives'),
        if (controller.lastError case final String error)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Text(error),
          ),
        Expanded(
          child: ObjectiveList(
            objectives: controller.objectives,
            selectedObjectiveId: controller.selectedObjectiveId,
            onSelected: controller.selectObjective,
          ),
        ),
      ],
    );
  }

  static Widget _timeline(
    StudioController controller,
    ObjectiveSummary? selected,
  ) {
    if (selected == null) {
      return const Center(child: Text('Select an objective to inspect its execution.'));
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        _PaneHeading(selected.description),
        Expanded(
          child: ExecutionTimeline(
            events: controller.events,
            objectiveId: selected.id,
          ),
        ),
      ],
    );
  }

  static Widget _evidence(
    StudioController controller,
    ObjectiveSummary? selected,
  ) {
    if (selected == null) {
      return const Center(child: Text('No objective selected.'));
    }
    return EvidencePanel(objective: selected, events: controller.events);
  }

  static ObjectiveSummary? _selectedObjective(StudioController controller) {
    final selectedId = controller.selectedObjectiveId;
    if (selectedId == null) {
      return null;
    }
    for (final objective in controller.objectives) {
      if (objective.id == selectedId) {
        return objective;
      }
    }
    return null;
  }

  static String _connectionLabel(StudioConnectionState state) {
    return switch (state) {
      StudioConnectionState.disconnected => 'Disconnected',
      StudioConnectionState.connecting => 'Connecting',
      StudioConnectionState.connected => 'Connected',
    };
  }
}

final class _PaneHeading extends StatelessWidget {
  const _PaneHeading(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: Text(text, style: Theme.of(context).textTheme.titleMedium),
    );
  }
}
