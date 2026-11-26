import { defineAgent, llm, voice } from '@livekit/agents';
import { z } from 'zod';

import { enqueueObjective } from './autodev-client.js';
import { loadConfig } from './config.js';
import { AGENT_INSTRUCTIONS, OBJECTIVE_TOOL_DESCRIPTION } from './policy.js';

export function createObjectiveTool(config = loadConfig()) {
  return llm.tool({
    name: 'enqueue_autodev_objective',
    description: OBJECTIVE_TOOL_DESCRIPTION,
    parameters: z.object({
      description: z.string().trim().min(1).max(4_000),
      branch: z.string().trim().min(1).max(200).optional(),
    }),
    onDuplicate: 'reject',
    execute: async ({ description, branch }) => {
      const receipt = await enqueueObjective(config, {
        description,
        ...(branch ? { branch } : {}),
      });
      return `Objective ${receipt.id} queued for ${receipt.repository} on ${receipt.branch}. Status: ${receipt.status}.`;
    },
  });
}

export function createAutodevAgent(config = loadConfig()) {
  return voice.Agent.create({
    instructions: AGENT_INSTRUCTIONS,
    tools: [createObjectiveTool(config)],
  });
}

export default defineAgent({
  entry: async (ctx) => {
    const config = loadConfig();
    await ctx.connect();

    const session = new voice.AgentSession({
      stt: config.sttModel,
      llm: config.llmModel,
      tts: config.ttsModel,
      maxToolSteps: 2,
    });
    const agent = createAutodevAgent(config);

    if (config.avatar) {
      const { AvatarSession } = await import('@livekit/agents-plugin-lemonslice');
      const avatar = new AvatarSession({
        apiKey: config.avatar.apiKey,
        ...(config.avatar.agentId ? { agentId: config.avatar.agentId } : {}),
        ...(config.avatar.agentImageUrl ? { agentImageUrl: config.avatar.agentImageUrl } : {}),
      });
      await avatar.start(session, ctx.room);
    }

    await session.start({ agent, room: ctx.room });
    await session.generateReply({
      instructions: 'Greet the user and explain that you can clarify and queue an AutoDev objective.',
    });
  },
});