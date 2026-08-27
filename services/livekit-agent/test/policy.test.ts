import assert from 'node:assert/strict';
import test from 'node:test';

import { AGENT_INSTRUCTIONS, OBJECTIVE_TOOL_DESCRIPTION } from '../src/policy.js';

test('agent policy requires explicit submission and preserves the authority boundary', () => {
  assert.match(AGENT_INSTRUCTIONS, /clearly asks to submit/);
  assert.match(AGENT_INSTRUCTIONS, /Never claim.*approved.*executed.*tested.*merged.*deployed/);
  assert.match(AGENT_INSTRUCTIONS, /repository.*fixed by operator configuration/);
  assert.match(OBJECTIVE_TOOL_DESCRIPTION, /does not approve or execute/);
});