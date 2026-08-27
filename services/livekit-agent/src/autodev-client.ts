import type { AgentConfig } from './config.js';

export interface ObjectiveInput {
  description: string;
  branch?: string;
}

export interface ObjectiveReceipt {
  id: string;
  repository: string;
  description: string;
  branch: string;
  status: string;
}

export type FetchLike = typeof fetch;

function boundedText(value: string, name: string, maximum: number): string {
  const trimmed = value.trim();
  if (!trimmed) throw new Error(`${name} must not be empty`);
  if (trimmed.length > maximum) throw new Error(`${name} exceeds ${maximum} characters`);
  if (trimmed.includes('\u0000')) throw new Error(`${name} contains a NUL character`);
  return trimmed;
}

export async function enqueueObjective(
  config: AgentConfig,
  input: ObjectiveInput,
  fetchImpl: FetchLike = fetch,
): Promise<ObjectiveReceipt> {
  const description = boundedText(input.description, 'description', 4_000);
  const requestedBranch = input.branch?.trim();
  const branch = requestedBranch
    ? boundedText(requestedBranch, 'branch', 200)
    : config.autodevBranch;

  const headers: Record<string, string> = {
    accept: 'application/json',
    'content-type': 'application/json',
  };
  if (config.autodevBearerToken) {
    headers.authorization = `Bearer ${config.autodevBearerToken}`;
  }

  const response = await fetchImpl(`${config.autodevUrl}/api/v1/objectives`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      repository: config.autodevRepository,
      description,
      ...(branch ? { branch } : {}),
    }),
    signal: AbortSignal.timeout(10_000),
  });

  const responseText = await response.text();
  if (!response.ok) {
    throw new Error(`AutoDev objective enqueue failed with HTTP ${response.status}`);
  }

  let payload: unknown;
  try {
    payload = JSON.parse(responseText);
  } catch {
    throw new Error('AutoDev returned invalid JSON');
  }
  if (!payload || typeof payload !== 'object') throw new Error('AutoDev returned an invalid objective');
  const value = payload as Record<string, unknown>;
  for (const field of ['id', 'repository', 'description', 'branch', 'status'] as const) {
    if (typeof value[field] !== 'string') throw new Error(`AutoDev objective is missing ${field}`);
  }

  return {
    id: value.id as string,
    repository: value.repository as string,
    description: value.description as string,
    branch: value.branch as string,
    status: value.status as string,
  };
}