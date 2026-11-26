import { ServerOptions, cli } from '@livekit/agents';
import { fileURLToPath } from 'node:url';

import { loadConfig } from './config.js';

const config = loadConfig();

cli.runApp(
  new ServerOptions({
    agent: fileURLToPath(new URL('./agent.js', import.meta.url)),
    agentName: config.agentName,
  }),
);