import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:autodev_studio/protocol/api_client.dart';
import 'package:autodev_studio/protocol/sse_client.dart';
import 'package:flutter_test/flutter_test.dart';

Map<String, Object?> fixture(String name) {
  final file = File(
    '${Directory.current.path}/../../protocols/public/v1/fixtures/$name',
  );
  return Map<String, Object?>.from(jsonDecode(file.readAsStringSync()) as Map);
}

Future<HttpServer> server(
  FutureOr<void> Function(HttpRequest request) handler,
) async {
  final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
  server.listen(handler);
  return server;
}

Uri baseUri(HttpServer server) => Uri.parse('http://127.0.0.1:${server.port}');

void main() {
  test('lists typed objective summaries', () async {
    final local = await server((request) {
      expect(request.uri.path, '/api/v1/objectives');
      request.response.headers.contentType = ContentType.json;
      request.response.write(jsonEncode([fixture('objective-summary.queued.json')]));
      request.response.close();
    });
    addTearDown(() => local.close(force: true));

    final api = AutoDevApi(baseUri: baseUri(local));
    addTearDown(api.close);

    final objectives = await api.listObjectives();
    expect(objectives, hasLength(1));
    expect(objectives.single.id, 'obj-0001');
    expect(objectives.single.status, 'queued');
  });

  test('creates an objective using untrusted public intent only', () async {
    final captured = Completer<Map<String, Object?>>();
    final local = await server((request) async {
      expect(request.uri.path, '/api/v1/objectives');
      expect(request.method, 'POST');
      final body = await utf8.decoder.bind(request).join();
      captured.complete(Map<String, Object?>.from(jsonDecode(body) as Map));
      request.response.statusCode = HttpStatus.accepted;
      request.response.headers.contentType = ContentType.json;
      request.response.write(jsonEncode(fixture('objective-summary.queued.json')));
      await request.response.close();
    });
    addTearDown(() => local.close(force: true));

    final api = AutoDevApi(baseUri: baseUri(local));
    addTearDown(api.close);

    final objective = await api.createObjective(
      repository: 'owner/repo',
      description: 'Implement health endpoint',
    );
    expect(objective.id, 'obj-0001');

    final request = await captured.future;
    expect(request['repository'], 'owner/repo');
    expect(request['description'], 'Implement health endpoint');
    expect(request.containsKey('approval_ref'), isFalse);
    expect(request.containsKey('capabilities'), isFalse);
  });

  test('decodes a typed SSE event and can abort the stream', () async {
    final connected = Completer<void>();
    final release = Completer<void>();
    final local = await server((request) async {
      expect(request.uri.path, '/events');
      request.response.headers.contentType = ContentType('text', 'event-stream');
      request.response.write(
        'data: ${jsonEncode(fixture('objective-event.queued.json'))}\n\n',
      );
      await request.response.flush();
      if (!connected.isCompleted) connected.complete();
      await release.future;
      await request.response.close();
    });
    addTearDown(() async {
      if (!release.isCompleted) release.complete();
      await local.close(force: true);
    });

    final events = AutoDevEventStream();
    final first = events.connect(baseUri(local).replace(path: '/events')).first;
    await connected.future;
    expect((await first).type, 'objective_queued');
    await events.close().timeout(const Duration(seconds: 1));
  });

  test('retries a recoverable SSE failure using bounded backoff', () async {
    var requests = 0;
    final delays = <Duration>[];
    final local = await server((request) async {
      requests += 1;
      if (requests == 1) {
        request.response.statusCode = HttpStatus.serviceUnavailable;
        await request.response.close();
        return;
      }
      request.response.headers.contentType = ContentType('text', 'event-stream');
      request.response.write(
        'data: ${jsonEncode(fixture('objective-event.queued.json'))}\n\n',
      );
      await request.response.close();
    });
    addTearDown(() => local.close(force: true));

    final events = AutoDevEventStream(
      delay: (duration) async {
        delays.add(duration);
      },
    );
    addTearDown(events.close);

    final event = await events.connect(baseUri(local).replace(path: '/events')).first;
    expect(event.objectiveId, 'obj-0001');
    expect(requests, 2);
    expect(delays, [const Duration(milliseconds: 250)]);
  });
}
