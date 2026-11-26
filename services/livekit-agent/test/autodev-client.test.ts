import assert from 'node:assert/strict';
import test from 'node:test';

import { enqueueObjective } from '../src/autodev-client.js';
import type { AgentConfig } from '../src/config.js';

const config: AgentConfig = {
  agentName: 'autodev-assistant',
  autodevUrl: 'http://127.0.0.1:8080',
  autodevRepository: 'owner/repo',
  autodevBearerToken: 'test-token',
  sttModel: 'stt',
  llmModel: 'llm',
  ttsModel: 'tts',
};

test('queues only the configured repository and sends bearer authentication', async () => {
  let request: Request | undefined;
  const receipt = await enqueueObjective(
    config,
    { description: ' Add a health endpoint ', branch: 'feature/health' },
    async (input, init) => {
      request = new Request(input, init);
      return Response.json(
        {
          id: 'objective-1',
          repository: 'owner/repo',
          description: 'Add a health endpoint',
          branch: 'feature/health',
          status: 'queued',
        },
        { status: 202 },
      );
    },
  );

  assert.equal(receipt.id, 'objective-1');
  assert.equal(request?.headers.get('authorization'), 'Bearer test-token');
  assert.deepEqual(await request?.json(), {
    repository: 'owner/repo',
    description: 'Add a health endpoint',
    branch: 'feature/health',
  });
});

test('rejects oversized model-supplied objectives before network access', async () => {
  let called = false;
  await assert.rejects(
    enqueueObjective(config, { description: 'x'.repeat(4_001) }, async () => {
      called = true;
      return Response.json({});
    }),
    /exceeds 4000/,
  );
  assert.equal(called, false);
});

test('does not return a remote error body that could contain secrets', async () => {
  await assert.rejects(
    enqueueObjective(config, { description: 'test' }, async () =>
      new Response('upstream-secret-detail', { status: 500 }),
    ),
    (error) => error instanceof Error && !error.message.includes('upstream-secret-detail'),
  );
});