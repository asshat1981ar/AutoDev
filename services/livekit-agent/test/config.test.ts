import assert from 'node:assert/strict';
import test from 'node:test';

import { loadConfig, normalizeAutodevUrl } from '../src/config.js';

const baseEnv = {
  LIVEKIT_AGENT_NAME: 'autodev-assistant',
  AUTODEV_REPOSITORY: 'owner/repo',
  AUTODEV_LIVEKIT_STT_MODEL: 'deepgram/nova-3',
  AUTODEV_LIVEKIT_LLM_MODEL: 'openai/gpt-4.1-mini',
  AUTODEV_LIVEKIT_TTS_MODEL: 'cartesia/sonic-3:test-voice',
};

test('defaults AutoDev access to loopback', () => {
  const config = loadConfig(baseEnv);
  assert.equal(config.autodevUrl, 'http://127.0.0.1:8080');
  assert.equal(config.autodevRepository, 'owner/repo');
  assert.equal(config.avatar, undefined);
});

test('rejects a remote control plane unless explicitly enabled', () => {
  assert.throws(
    () => loadConfig({ ...baseEnv, AUTODEV_URL: 'https://autodev.example.com' }),
    /AUTODEV_ALLOW_REMOTE=1/,
  );
  assert.throws(
    () => normalizeAutodevUrl('http://autodev.example.com', true),
    /must use https/,
  );
  assert.throws(
    () =>
      loadConfig({
        ...baseEnv,
        AUTODEV_URL: 'https://autodev.example.com',
        AUTODEV_ALLOW_REMOTE: '1',
      }),
    /AUTODEV_API_BEARER_TOKEN/,
  );
});

test('requires exactly one LemonSlice avatar source', () => {
  assert.throws(
    () => loadConfig({ ...baseEnv, LEMONSLICE_API_KEY: 'secret' }),
    /exactly one/,
  );
  assert.throws(
    () =>
      loadConfig({
        ...baseEnv,
        LEMONSLICE_API_KEY: 'secret',
        LEMONSLICE_AGENT_ID: 'agent-1',
        LEMONSLICE_AGENT_IMAGE_URL: 'https://example.com/avatar.png',
      }),
    /exactly one/,
  );
});

test('requires explicit dispatch and rejects unsafe avatar URLs', () => {
  const { LIVEKIT_AGENT_NAME: _, ...withoutAgentName } = baseEnv;
  assert.throws(() => loadConfig(withoutAgentName), /LIVEKIT_AGENT_NAME/);
  assert.throws(
    () =>
      loadConfig({
        ...baseEnv,
        LEMONSLICE_API_KEY: 'secret',
        LEMONSLICE_AGENT_IMAGE_URL: 'http://example.com/avatar.png',
      }),
    /must be an HTTPS URL/,
  );
});

test('does not include secret values in configuration errors', () => {
  const secret = 'do-not-leak-this-value';
  assert.throws(
    () =>
      loadConfig({
        ...baseEnv,
        LEMONSLICE_API_KEY: secret,
      }),
    (error) => error instanceof Error && !error.message.includes(secret),
  );
});