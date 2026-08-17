import 'models.dart';

abstract interface class ObjectiveRepository {
  Future<List<ObjectiveSummary>> listObjectives();

  Future<ObjectiveSummary> createObjective({
    required String repository,
    required String description,
    String? branch,
  });

  Future<void> close();
}

abstract interface class ObjectiveEventSource {
  Stream<ObjectiveEvent> connect(Uri endpoint);

  Future<void> close();
}
