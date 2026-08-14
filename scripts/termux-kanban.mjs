#!/usr/bin/env node
/**
 * AutoDev Termux compatibility launcher for Cline Kanban.
 *
 * On Android/Termux ARM64, upstream node-pty may be installed without a usable
 * Android native binding. This launcher probes the installed Kanban PTY first
 * and only replaces it when it cannot load. The replacement is pinned to an
 * Android ARM64 prebuilt and its native binary is SHA-256 verified.
 */
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const PTY_VERSION = '1.1.2';
const PTY_ALIAS = `node-pty@npm:@mmmbuto/node-pty-android-arm64@${PTY_VERSION}`;
const PTY_SHA256 = '660a3025230f6035b7b8c000e8cca6ca3992bedaa05f7b165e7c3a5f1ae8ec8a';
const NATIVE_RELATIVE_PATH = join('prebuilds', 'android-arm64', 'pty.node');

function log(message) {
  process.stderr.write(`[termux-kanban] ${message}\n`);
}

function fail(message, code = 1) {
  log(`ERROR: ${message}`);
  process.exit(code);
}

function command(commandName, args, options = {}) {
  return spawnSync(commandName, args, {
    encoding: 'utf8',
    ...options,
  });
}

function isTermuxArm64() {
  const prefix = process.env.PREFIX ?? '';
  const termux = process.platform === 'android' || prefix.includes('com.termux');
  return termux && process.arch === 'arm64';
}

function npmGlobalRoot() {
  const result = command('npm', ['root', '-g']);
  if (result.error) {
    fail(`cannot execute npm: ${result.error.message}`);
  }
  if (result.status !== 0) {
    fail(`npm root -g failed: ${(result.stderr || result.stdout || '').trim()}`);
  }
  const root = result.stdout.trim();
  if (!root) fail('npm returned an empty global module root');
  return root;
}

function findKanbanRoot() {
  const root = join(npmGlobalRoot(), 'kanban');
  if (!existsSync(join(root, 'package.json'))) {
    fail(`global Kanban package not found at ${root}. Install/update Cline Kanban first.`);
  }
  return root;
}

function readPackageVersion(packageRoot) {
  try {
    return JSON.parse(readFileSync(join(packageRoot, 'package.json'), 'utf8')).version ?? 'unknown';
  } catch {
    return 'unknown';
  }
}

function probeNodePty(nodePtyRoot) {
  if (!existsSync(nodePtyRoot)) {
    return { ok: false, detail: 'node_modules/node-pty is missing' };
  }
  const probe = [
    `const p = require(${JSON.stringify(nodePtyRoot)});`,
    `if (!p || typeof p.spawn !== 'function') { throw new Error('node-pty spawn export missing'); }`,
  ].join(' ');
  const result = command(process.execPath, ['-e', probe]);
  if (result.status === 0) return { ok: true, detail: 'native module loaded' };
  return {
    ok: false,
    detail: (result.stderr || result.stdout || `node exited ${result.status}`).trim(),
  };
}

function sha256(path) {
  const hash = createHash('sha256');
  hash.update(readFileSync(path));
  return hash.digest('hex');
}

function verifyPinnedBinary(nodePtyRoot) {
  const nativePath = join(nodePtyRoot, NATIVE_RELATIVE_PATH);
  if (!existsSync(nativePath)) {
    fail(`pinned package installed without ${NATIVE_RELATIVE_PATH}`);
  }
  const actual = sha256(nativePath);
  if (actual !== PTY_SHA256) {
    fail(`PTY binary checksum mismatch: expected ${PTY_SHA256}, got ${actual}`);
  }
}

function repairNodePty(kanbanRoot) {
  const nodePtyRoot = join(kanbanRoot, 'node_modules', 'node-pty');
  log(`repairing node-pty with ${PTY_ALIAS}`);
  rmSync(nodePtyRoot, { recursive: true, force: true });

  const result = command(
    'npm',
    [
      'install',
      '--omit=dev',
      '--no-save',
      '--package-lock=false',
      '--ignore-scripts',
      PTY_ALIAS,
    ],
    { cwd: kanbanRoot, stdio: 'inherit' },
  );
  if (result.error) fail(`npm repair failed to start: ${result.error.message}`);
  if (result.status !== 0) fail(`npm repair exited with code ${result.status}`);

  verifyPinnedBinary(nodePtyRoot);
  const probe = probeNodePty(nodePtyRoot);
  if (!probe.ok) fail(`repaired node-pty still cannot load: ${probe.detail}`);
  log('Android PTY repair verified');
}

function ensureCompatible({ forceRepair = false } = {}) {
  if (!isTermuxArm64()) {
    return { patched: false, skipped: true, reason: `${process.platform}/${process.arch}` };
  }

  const kanbanRoot = findKanbanRoot();
  const nodePtyRoot = join(kanbanRoot, 'node_modules', 'node-pty');
  const initial = probeNodePty(nodePtyRoot);

  if (initial.ok && !forceRepair) {
    log(`Kanban ${readPackageVersion(kanbanRoot)} PTY already loads; no repair needed`);
    return { patched: false, skipped: false, kanbanRoot };
  }

  if (!initial.ok) {
    const firstLine = initial.detail.split('\n')[0];
    log(`PTY probe failed: ${firstLine}`);
  } else {
    log('forced repair requested');
  }

  repairNodePty(kanbanRoot);
  return { patched: true, skipped: false, kanbanRoot };
}

function checkOnly() {
  if (!isTermuxArm64()) {
    log(`not an Android/Termux ARM64 runtime (${process.platform}/${process.arch}); no patch is required`);
    return 0;
  }
  const kanbanRoot = findKanbanRoot();
  const nodePtyRoot = join(kanbanRoot, 'node_modules', 'node-pty');
  const probe = probeNodePty(nodePtyRoot);
  log(`Kanban: ${kanbanRoot} (${readPackageVersion(kanbanRoot)})`);
  if (probe.ok) {
    log('PTY status: healthy');
    return 0;
  }
  log(`PTY status: broken (${probe.detail.split('\n')[0]})`);
  return 2;
}

function launchKanban(args) {
  let result = command('cline', ['kanban', ...args], { stdio: 'inherit' });
  if (result.error?.code === 'ENOENT') {
    log('cline executable not found; falling back to kanban');
    result = command('kanban', args, { stdio: 'inherit' });
  }
  if (result.error) fail(`unable to launch Kanban: ${result.error.message}`);
  if (result.signal) {
    log(`Kanban terminated by ${result.signal}`);
    return 128;
  }
  return result.status ?? 1;
}

function usage() {
  process.stdout.write(`Usage: node scripts/termux-kanban.mjs [adapter-options] [--] [kanban-options]\n\n`);
  process.stdout.write(`Adapter options:\n`);
  process.stdout.write(`  --check         Probe the currently installed PTY; do not repair or launch.\n`);
  process.stdout.write(`  --repair-only   Repair if required; do not launch Kanban.\n`);
  process.stdout.write(`  --force-repair  Reinstall the pinned Android PTY even if the probe succeeds.\n`);
  process.stdout.write(`  -h, --help      Show this help.\n\n`);
  process.stdout.write(`Any remaining arguments are passed to \`cline kanban\`.\n`);
}

function main() {
  const args = process.argv.slice(2);
  let check = false;
  let repairOnly = false;
  let forceRepair = false;

  while (args.length > 0) {
    if (args[0] === '--') {
      args.shift();
      break;
    }
    if (args[0] === '--check') {
      check = true;
      args.shift();
      continue;
    }
    if (args[0] === '--repair-only') {
      repairOnly = true;
      args.shift();
      continue;
    }
    if (args[0] === '--force-repair') {
      forceRepair = true;
      args.shift();
      continue;
    }
    if (args[0] === '-h' || args[0] === '--help') {
      usage();
      return 0;
    }
    break;
  }

  if (check) return checkOnly();

  const result = ensureCompatible({ forceRepair });
  if (result.skipped) {
    log(`compatibility repair skipped on ${result.reason}`);
  }
  if (repairOnly) return 0;
  return launchKanban(args);
}

process.exitCode = main();
