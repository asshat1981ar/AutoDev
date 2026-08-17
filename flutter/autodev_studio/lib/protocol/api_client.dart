import 'dart:convert';

import 'package:http/http.dart' as http;

import 'models.dart';

final class AutoDevHttpException implements Exception {
  const AutoDevHttpException(this.statusCode, this.message);

  final int statusCode;
  final String message;

  @override
  String toString() => 'AutoDevHttpException($statusCode, $message)';
}

final class AutoDevApi {
  AutoDevApi({required Uri baseUri, http.Client? client})
    : _baseUri = baseUri,
      _client = client ?? http.Client(),
      _ownsClient = client == null;

  final Uri _baseUri;
  final http.Client _client;
  final bool _ownsClient;
  bool _closed = false;

  Future<List<ObjectiveSummary>> listObjectives() async {
    _ensureOpen();
    final response = await _client.get(_endpoint('/api/v1/objectives'));
    _requireSuccess(response);
    final decoded = jsonDecode(response.body);
    if (decoded is! List) {
      throw const FormatException('objective list must be an array');
    }
    return List<ObjectiveSummary>.unmodifiable(
      decoded.map(
        (value) => ObjectiveSummary.fromJson(_objectMap(value, 'objective')),
      ),
    );
  }

  Future<ObjectiveSummary> createObjective({
    required String repository,
    required String description,
    String? branch,
  }) async {
    _ensureOpen();
    final payload = <String, Object?>{
      'repository': repository,
      'description': description,
      if (branch != null) 'branch': branch,
    };
    final response = await _client.post(
      _endpoint('/api/v1/objectives'),
      headers: const {'content-type': 'application/json'},
      body: jsonEncode(payload),
    );
    _requireSuccess(response);
    final decoded = jsonDecode(response.body);
    return ObjectiveSummary.fromJson(_objectMap(decoded, 'objective'));
  }

  Future<void> close() async {
    if (_closed) {
      return;
    }
    _closed = true;
    if (_ownsClient) {
      _client.close();
    }
  }

  Uri _endpoint(String path) {
    return _baseUri.replace(path: path, query: null, fragment: null);
  }

  void _ensureOpen() {
    if (_closed) {
      throw StateError('AutoDevApi is closed');
    }
  }

  static void _requireSuccess(http.Response response) {
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw AutoDevHttpException(response.statusCode, response.body);
    }
  }
}

Map<String, Object?> _objectMap(Object? value, String label) {
  if (value is! Map) {
    throw FormatException('$label must be an object');
  }
  try {
    return Map<String, Object?>.from(value);
  } on TypeError {
    throw FormatException('$label must have string keys');
  }
}
