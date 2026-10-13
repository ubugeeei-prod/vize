import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { writeFakeCommand } from "./fake-command.ts";
import { controlledPublishFixture } from "./publish-npm-package-dirs-fixture.ts";
import { releaseCommandPath } from "./release-command.ts";

// Hold successful active children until the actual parent reports its failure
// stop. The failing child waits for the entire initial wave to start. No timing
// assumption can turn a queued launch into an accepted active publisher.
export function failedPublishFixture(workers: number) {
  const fixture = controlledPublishFixture();
  const observed = path.join(path.dirname(fixture.receiptPath), "parent-observed-failure");
  writeFakeCommand(
    fixture.binDir,
    "fake-node",
    [
      "const fs = require('node:fs');",
      "const path = require('node:path');",
      "const { spawn } = require('node:child_process');",
      "const args = process.argv.slice(2);",
      "if (args[0]?.replaceAll('\\\\', '/').endsWith('/publish_npm_package/main.ts')) {",
      "  const packageName = path.basename(args[1]);",
      "  const record = phase => fs.appendFileSync(process.env.MOCK_PUBLISH_EVENTS, JSON.stringify({ phase, package: packageName, time: Date.now(), args: args.slice(2) }) + '\\n');",
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
      "  const child = spawn(process.env.REAL_NODE_BIN, args, { stdio: ['ignore', 'pipe', 'pipe'] });",
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
  const env = {
    ...fixture.env,
    MOCK_PUBLISH_BARRIER_COUNT: String(workers),
    MOCK_PUBLISH_PARENT_OBSERVED: observed,
  };
  return {
    ...fixture,
    env,
    failureObserved: () => fs.existsSync(observed),
    // Run the actual parent through the observing runtime so successful
    // children can wait until the parent reports its failure stop.
    runParent: (args: readonly string[]) => {
      const childEnv: NodeJS.ProcessEnv = { ...process.env, ...env };
      delete childEnv.NODE_TEST_CONTEXT;
      return spawnSync(env.NODE_BIN, [releaseCommandPath("publish_npm_package_dirs"), ...args], {
        env: childEnv,
        encoding: "utf8",
      });
    },
  };
}
