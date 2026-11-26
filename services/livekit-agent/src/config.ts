import { isIP } from 'node:net';

const DEFAULT_AUTODEV_URL = 'http://127.0.0.1:8080';

export interface AvatarConfig {
  apiKey: string;
  agentId?: string;
  agentImageUrl?: string;
}

export interface AgentConfig {
  agentName: string;
  autodevUrl: string;
  autodevRepository: string;
  autodevBranch?: string;
  autodevBearerToken?: string;
  sttModel: string;
  llmModel: string;
  ttsModel: string;
  avatar?: AvatarConfig;
}

function required(env: NodeJS.ProcessEnv, name: string): string {
  const value = env[name]?.trim();
  if (!value) throw new Error(`missing required environment variable ${name}`);
  return value;
}

function optional(env: NodeJS.ProcessEnv, name: string): string | undefined {
  const value = env[name]?.trim();
  return value || undefined;
}

function isLoopback(hostname: string): boolean {
  if (hostname === 'localhost') return true;
  const normalized = hostname.replace(/^\[|\]$/g, '');
  const family = isIP(normalized);
  if (family === 4) return normalized.startsWith('127.');
  if (family === 6) return normalized === '::1';
  return false;
}

export function normalizeAutodevUrl(value: string, allowRemote: boolean): string {
  const url = new URL(value);
  if (url.username || url.password) throw new Error('AUTODEV_URL must not contain credentials');
  if (url.search || url.hash) throw new Error('AUTODEV_URL must not contain a query or fragment');
  if (url.pathname !== '/' && url.pathname !== '') {
    throw new Error('AUTODEV_URL must be an origin without a path');
  }
  if (!['http:', 'https:'].includes(url.protocol)) {
    throw new Error('AUTODEV_URL must use http or https');
  }
  if (!isLoopback(url.hostname)) {
    if (!allowRemote) {
      throw new Error('remote AUTODEV_URL requires AUTODEV_ALLOW_REMOTE=1');
    }
    if (url.protocol !== 'https:') {
      throw new Error('remote AUTODEV_URL must use https');
    }
  }
  return url.origin;
}

function avatarConfig(env: NodeJS.ProcessEnv): AvatarConfig | undefined {
  const apiKey = optional(env, 'LEMONSLICE_API_KEY');
  const agentId = optional(env, 'LEMONSLICE_AGENT_ID');
  const agentImageUrl = optional(env, 'LEMONSLICE_AGENT_IMAGE_URL');
  const sources = [agentId, agentImageUrl].filter(Boolean);

  if (!apiKey && sources.length === 0) return undefined;
  if (!apiKey) throw new Error('LEMONSLICE_API_KEY is required when an avatar source is set');
  if (sources.length !== 1) {
    throw new Error(
      'exactly one of LEMONSLICE_AGENT_ID or LEMONSLICE_AGENT_IMAGE_URL is required',
    );
  }
  if (agentImageUrl) {
    const url = new URL(agentImageUrl);
    if (url.protocol !== 'https:' || url.username || url.password) {
      throw new Error('LEMONSLICE_AGENT_IMAGE_URL must be an HTTPS URL without credentials');
    }
  }

  return {
    apiKey,
    ...(agentId ? { agentId } : {}),
    ...(agentImageUrl ? { agentImageUrl } : {}),
  };
}

export function loadConfig(env: NodeJS.ProcessEnv = process.env): AgentConfig {
  const autodevRepository = required(env, 'AUTODEV_REPOSITORY');
  if (autodevRepository.length > 512) throw new Error('AUTODEV_REPOSITORY exceeds 512 characters');
  const autodevUrl = normalizeAutodevUrl(
    optional(env, 'AUTODEV_URL') ?? DEFAULT_AUTODEV_URL,
    env.AUTODEV_ALLOW_REMOTE === '1',
  );
  const autodevBranch = optional(env, 'AUTODEV_BRANCH');
  const autodevBearerToken = optional(env, 'AUTODEV_API_BEARER_TOKEN');
  const avatar = avatarConfig(env);
  if (!isLoopback(new URL(autodevUrl).hostname) && !autodevBearerToken) {
    throw new Error('remote AUTODEV_URL requires AUTODEV_API_BEARER_TOKEN');
  }

  return {
    agentName: required(env, 'LIVEKIT_AGENT_NAME'),
    autodevUrl,
    autodevRepository,
    ...(autodevBranch ? { autodevBranch } : {}),
    ...(autodevBearerToken ? { autodevBearerToken } : {}),
    sttModel: required(env, 'AUTODEV_LIVEKIT_STT_MODEL'),
    llmModel: required(env, 'AUTODEV_LIVEKIT_LLM_MODEL'),
    ttsModel: required(env, 'AUTODEV_LIVEKIT_TTS_MODEL'),
    ...(avatar ? { avatar } : {}),
  };
}