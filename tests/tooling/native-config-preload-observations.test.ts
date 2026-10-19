import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, mkdir, readFile, realpath, rm, symlink, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "node:test";

import {
  configModuleEvaluationTimings,
  importConfigModuleWithDependencies,
} from "../../crates/vize_carton/src/config/loader/module-runtime.mjs";

const require = createRequire(import.meta.url);
const vite = dirname(require.resolve("vite/package.json"));
const loader = new URL("../../crates/vize_carton/src/config/loader/js.rs", import.meta.url);
const rust = await readFile(loader, "utf8");
const embedded =
  (
    await Promise.all(
      [...rust.matchAll(/include_str!\("([^"]+)"\)/g)].map((match) =>
        readFile(new URL(match[1], loader), "utf8"),
      ),
    )
  ).join("") + [...rust.matchAll(/r#"([\s\S]*?)"#/g)][0][1];

async function project(run: (root: string, selected: string) => Promise<void>) {
  const root = await realpath(await mkdtemp(join(tmpdir(), "vize-config-preload-")));
  try {
    await writeFile(join(root, "package.json"), '{"type":"module"}');
    await run(root, join(root, "vite.config.ts"));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

test("actual Node loader candidates are observed before lstat and selected manifest/entry before loading", async () => {
  await project(async (root, selected) => {
    const candidate = join(root, "node_modules/vite");
    const counter = join(root, "factory.txt");
    await writeFile(
      selected,
      `import { appendFileSync } from 'node:fs';
       const value: number = 93;
       export default () => { appendFileSync(${JSON.stringify(counter)}, 'once\\n'); return { value }; };`,
    );
    const phases: string[][] = [];
    const loaded = await importConfigModuleWithDependencies(
      selected,
      undefined,
      undefined,
      async (namespace) => (namespace.default as () => unknown)(),
      undefined,
      async (paths) => {
        phases.push([...paths]);
        if (paths.includes(candidate)) {
          await mkdir(dirname(candidate), { recursive: true });
          await symlink(vite, candidate, "junction");
        }
        await assert.rejects(readFile(counter), { code: "ENOENT" });
      },
    );
    assert.deepEqual(loaded.value, { value: 93 });
    assert.deepEqual(phases[0], [candidate]);
    assert.deepEqual(phases[1], [join(candidate, "package.json"), join(vite, "package.json")]);
    const entry = require.resolve("vite");
    assert.deepEqual(phases[2], [entry, await realpath(entry)]);
    assert.deepEqual(phases[3], [selected]);
    assert.equal(await readFile(counter, "utf8"), "once\n");
  });
});

test("a broken-present loader installed at the actual candidate phase never falls through", async () => {
  await project(async (root, selected) => {
    await writeFile(selected, "throw new Error('authored module must not execute');");
    const candidate = join(root, "node_modules/vite");
    const phases: string[][] = [];
    await assert.rejects(
      importConfigModuleWithDependencies(
        selected,
        undefined,
        undefined,
        undefined,
        undefined,
        async (paths) => {
          phases.push([...paths]);
          if (paths.includes(candidate)) {
            await mkdir(candidate, { recursive: true });
            await writeFile(
              join(candidate, "package.json"),
              '{"name":"vite","exports":"./missing.mjs"}',
            );
          }
        },
      ),
      /missing\.mjs/,
    );
    assert.equal(phases.length, 2, "selection stops before engine/config loading");
    assert.deepEqual(phases[0], [candidate]);
    assert.deepEqual(phases[1], [join(candidate, "package.json"), join(candidate, "package.json")]);
  });
});

async function observedNative(
  selected: string,
  beforeRead: (paths: string[]) => Promise<boolean | void>,
  trace?: boolean,
) {
  const child = spawn(
    process.execPath,
    ["--input-type=module", "-e", embedded, selected, "--observe-config-inputs"],
    {
      cwd: dirname(selected),
      stdio: ["pipe", "pipe", "pipe"],
      ...(trace === undefined
        ? {}
        : { env: { ...process.env, VIZE_TRACE_EDITOR_PREPARATION: trace ? "1" : "0" } }),
    },
  );
  let stderr = "";
  child.stderr.setEncoding("utf8").on("data", (part) => {
    stderr += part;
  });
  const exited = new Promise<number | null>((resolve, reject) => {
    child.on("error", reject);
    child.on("close", resolve);
  });
  let buffered = Buffer.alloc(0);
  const output = child.stdout[Symbol.asyncIterator]();
  async function fill(length: number) {
    while (buffered.length < length) {
      const next = await output.next();
      assert.equal(next.done, false, `unexpected child EOF: ${stderr}`);
      buffered = Buffer.concat([buffered, next.value]);
    }
  }
  try {
    for (;;) {
      while (!buffered.includes(10)) await fill(buffered.length + 1);
      const newline = buffered.indexOf(10);
      const header = buffered.subarray(0, newline).toString();
      buffered = buffered.subarray(newline + 1);
      if (header === "R") break;
      assert.match(header, /^B\d+$/);
      const length = Number(header.slice(1));
      assert.ok(length <= 65536);
      await fill(length);
      const paths = JSON.parse(buffered.subarray(0, length).toString()) as string[];
      buffered = buffered.subarray(length);
      if ((await beforeRead(paths)) === false) child.stdin.end();
      else child.stdin.write(Buffer.from([6]));
    }
    child.stdin.end();
    for (;;) {
      const next = await output.next();
      if (next.done) break;
      buffered = Buffer.concat([buffered, next.value]);
    }
    return {
      status: await exited,
      stderr,
      payload: buffered,
      value: JSON.parse(buffered.toString()),
    };
  } finally {
    if (child.exitCode === null) child.kill();
    await exited;
  }
}

test("the single native child waits for each phase ACK and accepts an unbounded final config payload", async () => {
  await project(async (root, selected) => {
    const candidate = join(root, "node_modules/vite");
    const counter = join(root, "factory.txt");
    const source = (value: number) => `import { appendFileSync } from 'node:fs';
      const value: number = ${value}; export default () => {
        appendFileSync(${JSON.stringify(counter)}, 'once\\n');
        return { vue: { formatter: { printWidth: value, attributeGroups: [['x'.repeat(70000)]] } } };
      };`;
    await writeFile(selected, source(91));
    let configPhases = 0;
    const loaded = await observedNative(selected, async (paths) => {
      await assert.rejects(readFile(counter), { code: "ENOENT" });
      if (paths.includes(candidate)) {
        await mkdir(dirname(candidate), { recursive: true });
        await symlink(vite, candidate, "junction");
      }
      if (paths.includes(selected)) {
        configPhases++;
        await writeFile(selected, source(93));
      }
    });
    assert.equal(loaded.status, 0, loaded.stderr);
    assert.equal(configPhases, 1);
    assert.ok(loaded.payload.length > 65536);
    assert.equal(loaded.value.config.formatter.printWidth, 93);
    assert.equal(loaded.value.config.formatter.attributeGroups[0][0], "x".repeat(70000));
    assert.equal(await readFile(counter, "utf8"), "once\n");
  });
});

test("closing the observation ACK channel rejects the same child before authored evaluation", async () => {
  await project(async (root, selected) => {
    const counter = join(root, "factory.txt");
    await writeFile(
      selected,
      `import { writeFileSync } from 'node:fs'; writeFileSync(${JSON.stringify(counter)}, 'unexpected'); export default {};`,
    );
    const failed = await observedNative(selected, async () => false);
    assert.equal(failed.status, 1);
    assert.match(failed.stderr, /Config observation acknowledgement ended/);
    assert.equal("config" in failed.value, false);
    await assert.rejects(readFile(counter), { code: "ENOENT" });
  });
});

for (const installedVite of [false, true]) {
  test(`an authored exit before the native finally retains its actual status and diagnostic (Vite: ${installedVite})`, async () => {
    await project(async (root) => {
      if (installedVite) {
        await mkdir(join(root, "node_modules"));
        await symlink(vite, join(root, "node_modules/vite"), "junction");
      }
      const selected = join(root, "vite.config.mjs");
      const counter = join(root, "factory.txt");
      await writeFile(
        selected,
        `import { appendFileSync } from 'node:fs'; export default () => {
          appendFileSync(${JSON.stringify(counter)}, 'once\\n');
          console.error('authored-specificreason 日本語'); process.exit(17);
        };`,
      );
      const child = spawn(
        process.execPath,
        ["--input-type=module", "-e", embedded, selected, "--observe-config-inputs"],
        {
          cwd: root,
          stdio: ["pipe", "pipe", "pipe"],
        },
      );
      const exited = new Promise<number | null>((resolve, reject) => {
        child.on("error", reject);
        child.on("close", resolve);
      });
      const stderr: Buffer[] = [];
      child.stderr.on("data", (part: Buffer) => stderr.push(part));
      let buffered = Buffer.alloc(0);
      let phases = 0;
      try {
        for await (const chunk of child.stdout) {
          buffered = Buffer.concat([buffered, chunk]);
          for (;;) {
            const newline = buffered.indexOf(10);
            if (newline === -1) break;
            const header = buffered.subarray(0, newline).toString();
            assert.match(header, /^B\d+$/, "the process exit must bypass the final R/JSON frame");
            const length = Number(header.slice(1));
            if (buffered.length < newline + 1 + length) break;
            const paths = JSON.parse(
              buffered.subarray(newline + 1, newline + 1 + length).toString(),
            );
            assert.ok(paths.every((path: unknown) => typeof path === "string"));
            buffered = buffered.subarray(newline + 1 + length);
            phases++;
            child.stdin.write(Buffer.from([6]));
          }
        }
        assert.equal(await exited, 17);
        assert.equal(Buffer.concat(stderr).toString(), "authored-specificreason 日本語\n");
        assert.equal(buffered.length, 0);
        assert.ok(phases > 0);
        assert.equal(await readFile(counter, "utf8"), "once\n");
      } finally {
        child.stdin.end();
        if (child.exitCode === null) child.kill();
        await exited;
      }
    });
  });
}

test("real Vite failures carry only actual owned adapter identity while preserving the original diagnostic and repair", async () => {
  await project(async (root) => {
    const app = join(root, "space app 日本語");
    await mkdir(join(app, "node_modules"), { recursive: true });
    await symlink(vite, join(app, "node_modules/vite"), "junction");
    const selected = join(app, "vite.config.ts");
    const counter = join(app, "factory.txt");
    const source = (request: string) => `import vue from ${JSON.stringify(request)};
      import { appendFileSync } from 'node:fs';
      export default () => { appendFileSync(${JSON.stringify(counter)}, 'once\\n'); return { vue }; };`;
    await writeFile(selected, source("./missing-policy"));
    const attempts: Awaited<ReturnType<typeof observedNative>>[] = [];
    const invoke = async () => {
      let configLoads = 0;
      const result = await observedNative(selected, async (paths) => {
        if (paths.includes(selected)) configLoads++;
      });
      assert.equal(configLoads, 1, "each attempt enters its original config loader once");
      attempts.push(result);
      return result;
    };
    const projected = (result: Awaited<ReturnType<typeof observedNative>>) => {
      const mappings = result.value.failureArtifacts as Array<[string, string]>;
      assert.equal(mappings.length, 3);
      const [adapter, authored] = mappings[0];
      assert.equal(authored, selected);
      assert.ok(adapter.endsWith("/config.mjs"));
      assert.ok(dirname(adapter).split("/").at(-1)?.startsWith("vize-config-module-"));
      assert.deepEqual(mappings, [
        [adapter, selected],
        [pathToFileURL(adapter).href, pathToFileURL(selected).href],
        [relative(app, adapter), relative(app, selected)],
      ]);
      assert.ok(mappings.some(([owned]) => result.stderr.includes(owned)));
      assert.equal("config" in result.value, false);
      assert.equal(result.value.watchDependenciesComplete, false);
      for (const [owned] of mappings) {
        assert.equal(result.value.dependencies.includes(owned), false);
      }
      const raw = result.stderr;
      const identity = {
        status: result.status,
        stderr: [...mappings]
          .sort(([a], [b]) => b.length - a.length)
          .reduce((stderr, [owned, source]) => stderr.split(owned).join(source), raw),
      };
      assert.equal(result.stderr, raw, "identity metadata cannot replace original stderr");
      return identity;
    };
    const first = await invoke();
    const second = await invoke();
    assert.equal(first.status, 1);
    assert.equal(second.status, 1);
    assert.notDeepEqual(first.value.failureArtifacts, second.value.failureArtifacts);
    assert.notEqual(
      first.stderr,
      second.stderr,
      "actual fresh adapters remain visible in raw diagnostics",
    );
    assert.deepEqual(projected(first), projected(second));
    await assert.rejects(readFile(counter), { code: "ENOENT" });

    await writeFile(selected, source("./different-missing-policy"));
    const different = await invoke();
    assert.equal(different.status, 1);
    assert.notDeepEqual(
      projected(first),
      projected(different),
      "a distinct real authored failure stays visible",
    );
    await writeFile(
      join(app, "different-missing-policy.ts"),
      "export default { formatter: { printWidth: 93 } };",
    );
    const repaired = await invoke();
    assert.equal(repaired.status, 0, repaired.stderr);
    assert.equal(repaired.value.config.formatter.printWidth, 93);
    assert.equal("failureArtifacts" in repaired.value, false);
    assert.equal(await readFile(counter, "utf8"), "once\n");
    assert.equal(attempts.length, 4);
  });
});

test("private timing envelope preserves real Vite values, factory counts, ACK order and failure identity", async () => {
  await project(async (root, selected) => {
    await mkdir(join(root, "node_modules"));
    await symlink(vite, join(root, "node_modules/vite"), "junction");
    const counter = join(root, "timed-factory.txt");
    for (const failure of [false, true]) {
      await writeFile(
        selected,
        `import {appendFileSync} from 'node:fs';
        export default () => {
          appendFileSync(${JSON.stringify(counter)}, 'once\\n');
          ${failure ? "throw Object.assign(new Error('timed authored failure'), {stack:'Error: timed authored failure\\n    at authored timing fixture'});" : "return {vue:{languageServer:{hover:false}}};"}
        };`,
      );
      const outcomes: Array<
        Pick<Awaited<ReturnType<typeof observedNative>>, "status" | "value" | "stderr"> & {
          phases: string[][];
        }
      > = [];
      for (const trace of [false, true]) {
        const phases: string[][] = [];
        const result = await observedNative(
          selected,
          async (paths) => {
            phases.push(paths);
          },
          trace,
        );
        const timings = result.value.configEvaluationTimings;
        if (!trace) assert.equal(timings, undefined);
        else {
          const split = timings.filter((row: { phase: string }) =>
            row.phase.startsWith("public_load_"),
          );
          if (split.length) {
            assert.deepEqual(
              split.map((row: { phase: string }) => row.phase),
              ["public_load_until_owned_resolve", "public_load_after_owned_resolve"],
            );
            const load = timings.find(
              (row: { phase: string }) => row.phase === "public_load",
            ).elapsedMs;
            assert.equal(
              Math.abs(split[0].elapsedMs + split[1].elapsedMs - load) <=
                Number.EPSILON * Math.max(1, load) * 4,
              true,
            );
          }
          assert.deepEqual(
            timings
              .filter((row: { phase: string }) => !row.phase.startsWith("public_load_"))
              .map((row: { phase: string }) => row.phase),
            [
              "engine_selection_with_ack",
              "engine_import",
              "adapter_prepare",
              "selected_before_read_ack",
              "public_load",
              "export_evaluation",
              "adapter_cleanup",
              "module_total",
            ],
          );
          const profile = timings.find(
            (row: { phase: string }) => row.phase === "public_load",
          ).cpuProfile;
          assert.equal(profile.status, "recorded");
          assert.ok(Array.isArray(profile.profile.nodes) && profile.profile.nodes.length > 0);
          assert.ok(
            Array.isArray(profile.profile.samples) && Array.isArray(profile.profile.timeDeltas),
          );
          assert.equal(profile.profile.samples.length, profile.profile.timeDeltas.length);
          assert.ok(profile.setupMs >= 0 && profile.stopMs >= 0);
          const total = timings.at(-1).elapsedMs;
          for (const row of timings) {
            assert.equal(typeof row.elapsedMs, "number");
            assert.equal(Number.isSafeInteger(row.processId) && row.processId > 0, true);
            assert.equal(row.processId, timings[0].processId);
            assert.equal(row.elapsedMs <= total, true);
            assert.equal(Number.isFinite(row.elapsedMs) && row.elapsedMs >= 0, true);
          }
          delete result.value.configEvaluationTimings;
        }
        const mappings = (result.value.failureArtifacts ?? []) as Array<[string, string]>;
        const stderr = [...mappings]
          .sort(([left], [right]) => right.length - left.length)
          .reduce((text, [owned, source]) => text.split(owned).join(source), result.stderr);
        delete result.value.failureArtifacts;
        outcomes.push({ status: result.status, value: result.value, phases, stderr });
      }
      assert.deepEqual(outcomes[1], outcomes[0]);
      assert.equal(outcomes[0].status, failure ? 1 : 0);
      if (failure) {
        assert.match(outcomes[0].stderr, /timed authored failure/);
        const changed = {
          ...outcomes[1],
          stderr: outcomes[1].stderr.replaceAll(
            "timed authored failure",
            "different authored failure",
          ),
        };
        assert.throws(() => assert.deepEqual(changed, outcomes[0]), { code: "ERR_ASSERTION" });
      } else assert.equal(outcomes[0].value.config.languageServer.hover, false);
    }
    assert.equal(await readFile(counter, "utf8"), "once\nonce\nonce\nonce\n");
  });
});

test("public-load split stays UNKNOWN when no owned resolve was observed", async () => {
  await project(async (root, selected) => {
    await mkdir(join(root, "node_modules"));
    await symlink(vite, join(root, "node_modules/vite"), "junction");
    await writeFile(selected, "export default {value:93};");
    const previous = process.env.VIZE_TRACE_EDITOR_PREPARATION;
    process.env.VIZE_TRACE_EDITOR_PREPARATION = "1";
    try {
      const loaded = await importConfigModuleWithDependencies(selected, () => ({
        deregister() {},
      }));
      assert.deepEqual(loaded.namespace.default, { value: 93 });
      const rows = configModuleEvaluationTimings(loaded)!;
      assert.equal(rows.filter((row) => row.phase === "public_load").length, 1);
      assert.deepEqual(
        rows.filter((row) => row.phase.startsWith("public_load_")),
        [],
      );
    } finally {
      if (previous === undefined) delete process.env.VIZE_TRACE_EDITOR_PREPARATION;
      else process.env.VIZE_TRACE_EDITOR_PREPARATION = previous;
    }
  });
});
