import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;

import 'models.dart';

typedef Delay = Future<void> Function(Duration duration);

final class AutoDevEventStream {
  AutoDevEventStream({http.Client? client, Delay? delay})
    : _client = client ?? http.Client(),
      _ownsClient = client == null,
      _delay = delay ?? Future<void>.delayed;

  static const List<Duration> _retryDelays = <Duration>[
    Duration(milliseconds: 250),
    Duration(milliseconds: 500),
    Duration(seconds: 1),
    Duration(seconds: 2),
    Duration(seconds: 5),
  ];

  final http.Client _client;
  final bool _ownsClient;
  final Delay _delay;
  final Completer<void> _closeSignal = Completer<void>();

  Completer<void>? _activeAbort;
  Future<void>? _runner;
  bool _started = false;
  bool _closed = false;

  Stream<ObjectiveEvent> connect(Uri endpoint) {
    if (_started) {
      throw StateError('AutoDevEventStream.connect may only be called once');
    }
    if (_closed) {
      throw StateError('AutoDevEventStream is closed');
    }
    _started = true;

    final controller = StreamController<ObjectiveEvent>();
    _runner = _run(endpoint, controller).whenComplete(controller.close);
    return controller.stream;
  }

  Future<void> close() async {
    if (_closed) {
      await _runner;
      return;
    }
    _closed = true;
    if (!_closeSignal.isCompleted) {
      _closeSignal.complete();
    }
    final abort = _activeAbort;
    if (abort != null && !abort.isCompleted) {
      abort.complete();
    }
    if (_ownsClient) {
      _client.close();
    }
    await _runner;
  }

  Future<void> _run(
    Uri endpoint,
    StreamController<ObjectiveEvent> controller,
  ) async {
    var retryIndex = 0;
    while (!_closed) {
      final abort = Completer<void>();
      _activeAbort = abort;
      try {
        final request = http.AbortableRequest(
          'GET',
          endpoint,
          abortTrigger: abort.future,
        );
        request.headers['accept'] = 'text/event-stream';
        final response = await _client.send(request);
        if (response.statusCode < 200 || response.statusCode >= 300) {
          throw http.ClientException(
            'SSE returned HTTP ${response.statusCode}',
            endpoint,
          );
        }
        retryIndex = 0;
        await _consume(response.stream, controller);
      } on http.RequestAbortedException {
        if (!_closed) {
          await _retry(retryIndex++);
        }
        continue;
      } on Object {
        if (!_closed) {
          await _retry(retryIndex++);
        }
        continue;
      } finally {
        if (identical(_activeAbort, abort)) {
          _activeAbort = null;
        }
      }

      if (!_closed) {
        await _retry(retryIndex++);
      }
    }
  }

  Future<void> _consume(
    Stream<List<int>> bytes,
    StreamController<ObjectiveEvent> controller,
  ) async {
    var pending = '';
    await for (final text in bytes.transform(utf8.decoder)) {
      if (_closed) {
        return;
      }
      pending += text;
      while (true) {
        final boundary = _frameBoundary(pending);
        if (boundary == null) {
          break;
        }
        final frame = pending.substring(0, boundary.index);
        pending = pending.substring(boundary.index + boundary.length);
        _emitFrame(frame, controller);
      }
    }
    if (!_closed && pending.trim().isNotEmpty) {
      _emitFrame(pending, controller);
    }
  }

  void _emitFrame(
    String frame,
    StreamController<ObjectiveEvent> controller,
  ) {
    final dataLines = <String>[];
    for (final rawLine in const LineSplitter().convert(frame)) {
      final line = rawLine.endsWith('\r')
          ? rawLine.substring(0, rawLine.length - 1)
          : rawLine;
      if (line.startsWith('data:')) {
        dataLines.add(line.substring(5).trimLeft());
      }
    }
    if (dataLines.isNotEmpty) {
      _emitData(dataLines.join('\n'), controller);
    }
  }

  void _emitData(
    String encoded,
    StreamController<ObjectiveEvent> controller,
  ) {
    try {
      final decoded = jsonDecode(encoded);
      if (decoded is! Map) {
        throw const FormatException('SSE data must be a JSON object');
      }
      controller.add(
        ObjectiveEvent.fromJson(Map<String, Object?>.from(decoded)),
      );
    } on Object catch (error, stackTrace) {
      controller.addError(error, stackTrace);
    }
  }

  Future<void> _retry(int retryIndex) async {
    if (_closed) {
      return;
    }
    final bounded = retryIndex.clamp(0, _retryDelays.length - 1);
    await Future.any<void>(<Future<void>>[
      _delay(_retryDelays[bounded]),
      _closeSignal.future,
    ]);
  }
}

final class _FrameBoundary {
  const _FrameBoundary(this.index, this.length);

  final int index;
  final int length;
}

_FrameBoundary? _frameBoundary(String value) {
  final lf = value.indexOf('\n\n');
  final crlf = value.indexOf('\r\n\r\n');
  if (lf < 0 && crlf < 0) {
    return null;
  }
  if (lf >= 0 && (crlf < 0 || lf < crlf)) {
    return _FrameBoundary(lf, 2);
  }
  return _FrameBoundary(crlf, 4);
}
