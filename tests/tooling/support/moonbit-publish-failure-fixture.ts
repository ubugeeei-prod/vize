import fs from "node:fs";
import path from "node:path";

import { writeFakeCommand } from "./fake-command.ts";
import { controlledPublishFixture } from "./moonbit-publish-concurrency-fixture.ts";

// Hold successful active children until the actual parent reports its failure
// stop. The failing child waits for the entire initial wave to start. No timing
// assumption can turn a queued launch into an accepted active publisher.
export function failedPublishFixture(workers: number) {
  const fixture = controlledPublishFixture();
  const observed = path.join(path.dirname(fixture.receiptPath), "parent-observed-failure");
  writeFakeCommand(
    fixture.binDir,
    "moon",
    [
      "const fs = require('node:fs');",
      "const path = require('node:path');",
      "const { spawn } = require('node:child_process');",
      "const args = process.argv.slice(2);",
      "const separator = args.indexOf('--');",
      "if (args[separator - 1]?.replaceAll('\\\\', '/').endsWith('/publish_npm_package')) {",
      "  const packageName = path.basename(args[separator + 1]);",
      "  const record = phase => fs.appendFileSync(process.env.MOCK_PUBLISH_EVENTS, JSON.stringify({ phase, package: packageName, time: Date.now(), args: args.slice(separator + 2) }) + '\\n');",
      "  const started = () => fs.readFileSync(process.env.MOCK_PUBLISH_EVENTS, 'utf8').trim().split('\\n').map(JSON.parse).filter(event => event.phase === 'start').length;",
      "  record('start');",
      "  const deadline = Date.now() + 10000;",
      "  function poll() {",
      "    const failed = packageName === 'pkg-0';",
      "    const ready = failed ? started() >= Number(process.env.MOCK_PUBLISH_BARRIER_COUNT) : fs.existsSync(process.env.MOCK_PUBLISH_PARENT_OBSERVED);",
      "    if (!ready && Date.now() < deadline) { setTimeout(poll, 5); return; }",
      "    process.stdout.write('\\r\\npublisher stdout ' + packageName + '\\n\\n');",
      "    process.stderr.write('  publisher stderr ' + packageName + '\\r\\n\\r\\n');",
      "    record('end');",
      "    process.exit(ready ? (failed ? 7 : 0) : 89);",
      "  }",
      "  poll();",
      "} else {",
      "  const child = spawn(process.env.REAL_MOON_BIN, args, { stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, MOON_BIN: process.env.MOCK_CHILD_MOON_BIN ?? process.env.MOON_BIN } });",
      "  let stderr = '';",
      "  child.stdout.on('data', bytes => process.stdout.write(bytes));",
      "  child.stderr.on('data', bytes => {",
      "    process.stderr.write(bytes);",
      "    stderr += bytes.toString();",
      "    if (stderr.includes('Stopping queued package launches after publisher failure:')) fs.writeFileSync(process.env.MOCK_PUBLISH_PARENT_OBSERVED, 'actual parent observed failure');",
      "  });",
      "  child.on('error', error => { console.error(error); process.exitCode = 1; });",
      "  child.on('close', code => { process.exitCode = code ?? 1; });",
      "}",
    ].join("\n"),
  );
  return {
    ...fixture,
    env: {
      ...fixture.env,
      MOCK_PUBLISH_BARRIER_COUNT: String(workers),
      MOCK_PUBLISH_PARENT_OBSERVED: observed,
    },
    failureObserved: () => fs.existsSync(observed),
  };
}
