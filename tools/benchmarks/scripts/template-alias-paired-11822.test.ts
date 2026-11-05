import assert from "node:assert/strict";
import { test, type TestContext } from "node:test";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
  symlinkSync,
  writeSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  decodeFrames,
  diagnostics,
  metrics,
  resourceUsage,
  deriveConfig,
  installedNative,
  inventory,
  closureSnapshot,
  childEnvironment,
} from "./template-alias-paired-11822-custody.ts";
import {
  collectTransforms,
  directiveAuthority,
  admitSharedTransform,
} from "./template-alias-paired-11822-protocol.ts";
import {
  copyPublishedPackage,
  stageRuntimeDependencies,
  manifestChanges,
} from "./template-alias-paired-11822-packages.ts";
import {
  campaignBudget,
  recordedStage,
  remainingBudget,
} from "./template-alias-paired-11822-process.ts";
import type { ProtocolMessage } from "./template-alias-paired-11822-types.ts";

const scratch = (t: TestContext) => {
  const root = mkdtempSync(join(tmpdir(), "template-alias-law-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  return root;
};

void test("stage argv and raw streams survive timeout, with an explicit shared soft deadline", (t) => {
  const root = scratch(t),
    id = "interrupted-build";
  const started = 1000,
    deadline = started + campaignBudget.softMinutes * 60_000;
  const environment = {
    TEMPLATE_ALIAS_STARTED_MS: String(started),
    TEMPLATE_ALIAS_DEADLINE_MS: String(deadline),
  };
  assert.equal(remainingBudget(environment, deadline - 500), 500);
  assert.throws(() => remainingBudget(environment, deadline), /exhausted/);
  assert.throws(() => remainingBudget({}, 1000), /explicit/);
  assert.throws(
    () =>
      remainingBudget(
        { ...environment, TEMPLATE_ALIAS_DEADLINE_MS: String(deadline + 1) },
        started,
      ),
    /approved processing window/,
  );
  assert.throws(
    () =>
      recordedStage({
        directory: root,
        id,
        command: "native-build-fixture",
        args: ["--profile", "ci-opt"],
        cwd: root,
        sourceSha: "a".repeat(40),
        environment: { PATH: "/bin" },
        maxMilliseconds: 1000,
        budget: () => 500,
        run: (command, args, options) => {
          const started = JSON.parse(readFileSync(join(root, id + ".process.json"), "utf8")) as {
            command: string;
            ended: string | null;
            sourceSha: string;
          };
          assert.equal(started.command, "native-build-fixture");
          assert.equal(started.ended, null);
          assert.equal(started.sourceSha, "a".repeat(40));
          assert.equal(command, "/usr/bin/timeout");
          assert.ok(args.includes("0.5s"));
          assert.ok(Array.isArray(options.stdio));
          const stdout = options.stdio[1],
            stderr = options.stdio[2];
          assert.equal(typeof stdout, "number");
          assert.equal(typeof stderr, "number");
          writeSync(stdout as number, "partial stdout\n");
          writeSync(stderr as number, "partial stderr\n");
          return {
            pid: 1,
            status: 124,
            signal: null,
            output: [null, null, null],
            stdout: Buffer.alloc(0),
            stderr: Buffer.alloc(0),
          };
        },
      }),
    /timed out/,
  );
  assert.equal(readFileSync(join(root, id + ".stdout"), "utf8"), "partial stdout\n");
  assert.equal(readFileSync(join(root, id + ".stderr"), "utf8"), "partial stderr\n");
  const ended = JSON.parse(readFileSync(join(root, id + ".process.json"), "utf8")) as {
    status: number;
    timedOut: boolean;
    ended: string | null;
  };
  assert.equal(ended.status, 124);
  assert.equal(ended.timedOut, true);
  assert.ok(ended.ended);
});

void test("closed inventories reject external links and observe mutation of every admitted target and ignored image", (t) => {
  const root = scratch(t);
  const provider = join(root, "provider"),
    outside = join(root, "outside");
  mkdirSync(provider);
  mkdirSync(outside);
  writeFileSync(join(outside, "checker.js"), "before");
  symlinkSync(outside, join(provider, "dependency"));
  assert.throws(() => inventory(provider), /external symlink/);
  const before = closureSnapshot([provider, outside]);
  writeFileSync(join(outside, "checker.js"), "after");
  assert.notDeepEqual(closureSnapshot([provider, outside]), before);
  const image = join(root, "ignored-target.so");
  writeFileSync(image, "image-before");
  const imageBefore = closureSnapshot([provider, outside], [image]);
  writeFileSync(image, "image-after");
  assert.notDeepEqual(closureSnapshot([provider, outside], [image]), imageBefore);
});

void test("external processes never inherit CI credentials or override native resolution", () => {
  const result = childEnvironment(
    {},
    {
      PATH: "/bin",
      HOME: "/home/runner",
      GITHUB_TOKEN: "secret",
      GH_TOKEN: "secret",
      ACTIONS_RUNTIME_TOKEN: "secret",
      ACTIONS_ID_TOKEN_REQUEST_URL: "secret",
      NAPI_RS_NATIVE_LIBRARY_PATH: "/wrong.node",
    },
  );
  assert.deepEqual(result, { PATH: "/bin", HOME: "/home/runner" });
});

void test("normal packages keep wrapper and generated SDK files with a cycle-safe closed runtime graph", (t) => {
  const root = scratch(t),
    original = join(root, "source"),
    capsule = join(root, "capsule");
  mkdirSync(join(original, "dist"), { recursive: true });
  mkdirSync(join(original, "node_modules/b"), { recursive: true });
  writeFileSync(
    join(original, "package.json"),
    JSON.stringify({ name: "vize", files: ["dist"], dependencies: { b: "1" } }),
  );
  writeFileSync(join(original, "dist/cli.mjs"), "normal-wrapper");
  writeFileSync(
    join(original, "node_modules/b/package.json"),
    JSON.stringify({ name: "b", dependencies: { vize: "1" } }),
  );
  mkdirSync(join(original, "node_modules/b/node_modules"));
  symlinkSync(original, join(original, "node_modules/b/node_modules/vize"));
  const destination = join(capsule, "node_modules/vize");
  copyPublishedPackage(original, destination);
  stageRuntimeDependencies(original, destination, capsule, { vize: destination });
  assert.ok(
    closureSnapshot([capsule]).directories[0].files.some(({ path }) =>
      path.endsWith("dist/cli.mjs"),
    ),
  );
  assert.deepEqual(
    manifestChanges({ dependencies: { x: "workspace:*" } }, { dependencies: { x: "1.0.0" } }).map(
      ({ key }) => key,
    ),
    ["dependencies"],
  );
  assert.throws(() => manifestChanges({ name: "a" }, { name: "b" }), /non-dependency/);
});

void test("shared helper admission uses the actual SDK export set and preserves authored directive authority", () => {
  const text =
    'import type {} from "vize/virtual-ts-checker";\ntype T = import("vize/virtual-ts").Args;';
  assert.equal(admitSharedTransform(text, { virtualExports: ["Args"] }).mode, "shared-hoisted");
  assert.throws(() => admitSharedTransform(text, { virtualExports: ["Wrong"] }), /SDK/);
  assert.throws(
    () =>
      admitSharedTransform(text.replace('import type {} from "vize/virtual-ts-checker";', ""), {
        virtualExports: ["Args"],
      }),
    /inline helper/,
  );
  const packet = {
    directives: [[0, 1, 2, 3, 1, 0]],
    unusedExpectDirectiveDiagnostics: [{ code: 2578, messageText: "unused" }],
  };
  const before = directiveAuthority(packet, 10, 10);
  assert.deepEqual(
    directiveAuthority({ ...packet, directives: [[0, 1, 4, 8, 1, 0]] }, 10, 10),
    before,
  );
  assert.notDeepEqual(
    directiveAuthority({ ...packet, directives: [[1, 1, 4, 8, 1, 0]] }, 10, 10),
    before,
  );
  assert.throws(
    () => directiveAuthority({ ...packet, directives: [[0, 1, 9, 11, 1, 0]] }, 10, 10),
    /virtual range/,
  );
});

void test("transform custody rejects substituted paths, duplicates, missing and error responses", (t) => {
  const root = scratch(t),
    workspace = join(root, "work"),
    capture = join(root, "capture");
  mkdirSync(join(workspace, "src"), { recursive: true });
  mkdirSync(capture);
  for (const file of ["a.vue", "b.vue", "other.vue"])
    writeFileSync(join(workspace, "src", file), "source");
  const frame = (value: unknown) => {
    const bytes = Buffer.from(JSON.stringify(value));
    return Buffer.concat([Buffer.from(`Content-Length: ${bytes.length}\r\n\r\n`), bytes]);
  };
  const request = (id: number, file: string) => ({
    id,
    method: "transform",
    params: { fileName: join(workspace, "src", file), content: "source" },
  });
  const response = (id: number) => ({ id, result: { text: "transformed", extension: ".tsx" } });
  const run = (id: string, requests: ProtocolMessage[], responses: ProtocolMessage[]) => {
    writeFileSync(join(capture, "in-1.bin"), Buffer.concat(requests.map(frame)));
    writeFileSync(join(capture, "out-1.bin"), Buffer.concat(responses.map(frame)));
    return collectTransforms(capture, workspace, join(root, id), ["src/a.vue", "src/b.vue"]);
  };
  assert.equal(
    run("valid", [request(1, "a.vue"), request(2, "b.vue")], [response(1), response(2)]).count,
    2,
  );
  assert.throws(
    () =>
      run(
        "substituted",
        [request(1, "a.vue"), request(2, "other.vue")],
        [response(1), response(2)],
      ),
    /unexpected authored/,
  );
  assert.throws(
    () => run("missing", [request(1, "a.vue"), request(2, "b.vue")], [response(1)]),
    /unanswered/,
  );
  assert.throws(
    () =>
      run(
        "duplicate",
        [request(1, "a.vue"), request(2, "b.vue")],
        [response(1), response(1), response(2)],
      ),
    /duplicate transform response/,
  );
  assert.throws(
    () =>
      run(
        "error",
        [request(1, "a.vue"), request(2, "b.vue")],
        [response(1), { id: 2, error: { code: 1 } }],
      ),
    /failed transform/,
  );
});

void test("derivative configs preserve every original scope and mapper option", () => {
  const original = {
    extends: "./tsconfig.base.json",
    files: ["src/main.ts"],
    include: ["src/**/*.vue"],
    exclude: ["ignored"],
    compilerOptions: { strict: true, types: [], incremental: true },
    contentMappers: [
      {
        package: "@vue/content-mapper",
        extensions: [".vue", ".custom"],
        options: { strictTemplates: true },
      },
    ],
  };
  const before = structuredClone(original);
  const derived = deriveConfig(original, "@vue/content-mapper");
  assert.deepEqual(original, before);
  assert.deepEqual(derived, {
    ...before,
    compilerOptions: { ...before.compilerOptions, incremental: false },
    contentMappers: [{ ...before.contentMappers[0], package: "template-alias-vue" }],
  });
  assert.throws(() => deriveConfig(original, "vize"));
});

void test("native custody requires exact package version, SRI and real Linux x64 ELF", (t) => {
  const root = mkdtempSync(join(tmpdir(), "template-alias-custody-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const packageRoot = join(root, "node_modules/@tsrs/linux-x64-gnu");
  mkdirSync(packageRoot, { recursive: true });
  writeFileSync(
    join(packageRoot, "package.json"),
    JSON.stringify({ name: "@tsrs/linux-x64-gnu", version: "0.21.0-ts7.1.0-beta" }),
  );
  writeFileSync(
    join(root, "pnpm-lock.yaml"),
    "  '@tsrs/linux-x64-gnu@0.21.0-ts7.1.0-beta':\n    resolution: {integrity: sha512-example}\n",
  );
  const elf = Buffer.alloc(20);
  Buffer.from([0x7f, 0x45, 0x4c, 0x46, 2, 1]).copy(elf);
  elf.writeUInt16LE(62, 18);
  writeFileSync(join(packageRoot, "tsrs"), elf);
  const native = () =>
    installedNative(root, inventory(root), "@tsrs/linux-x64-gnu", "0.21.0-ts7.1.0-beta", ["tsrs"]);
  assert.equal(native().integrity, "sha512-example");
  writeFileSync(join(packageRoot, "vize-vitrine.linux-x64-gnu.node"), elf);
  assert.equal(
    installedNative(
      root,
      inventory(root),
      "@tsrs/linux-x64-gnu",
      "0.21.0-ts7.1.0-beta",
      ["vize-vitrine.linux-x64-gnu.node"],
      "napi",
    ).kind,
    "napi",
  );
  assert.throws(
    () =>
      installedNative(root, inventory(root), "@tsrs/linux-x64-gnu", "0.21.0-ts7.1.0-beta", [
        "vize-vitrine.linux-x64-gnu.node",
      ]),
    /not executable/,
  );
  elf.writeUInt16LE(183, 18);
  writeFileSync(join(packageRoot, "tsrs"), elf);
  assert.throws(native, /x86-64/);
  elf.writeUInt16LE(62, 18);
  writeFileSync(join(packageRoot, "tsrs"), elf);
  writeFileSync(join(root, "pnpm-lock.yaml"), "");
  assert.throws(native, /absent from pinned lock/);
});

const expected = Array.from({ length: 12 }, (_, n) => ({
  file: `src/generated/GeneratedComponent${String(n).padStart(4, "0")}.vue`,
  line: 40,
  column: n + 1,
  code: 2322,
}));
const output = expected
  .map(
    (d) =>
      `/work/${d.file}(${d.line},${d.column}): error TS${d.code}: Wrong value ${d.column}.\n  Detail ${d.column}.`,
  )
  .join("\n");

void test("parity retains complete multiline messages and rejects duplicate, extra and unparsed errors", () => {
  const packet = diagnostics(output, "/work", expected);
  assert.equal(packet.length, 12);
  assert.equal(packet[0].message, "Wrong value 1.\n  Detail 1.");
  assert.throws(
    () => diagnostics(output + "\n" + output.split("\n")[0], "/work", expected),
    /twelve/,
  );
  assert.throws(
    () => diagnostics(output + "\nerror TS6142: Missing JSX mode", "/work", expected),
    /unparsed/,
  );
  assert.throws(
    () => diagnostics(output.replace("TS2322", "TS2345"), "/work", expected),
    /locations/,
  );
  assert.throws(
    () => diagnostics(output.split("\n").slice(2).join("\n"), "/work", expected),
    /twelve/,
  );
  assert.notDeepEqual(
    packet,
    diagnostics(output.replace("Wrong value 1.", "Changed message."), "/work", expected),
  );
});

void test("protocol custody retains Unicode, directives and all bytes, refusing incomplete or ambiguous frames", () => {
  const message = {
    id: 1,
    result: {
      text: "const name = '日本語';",
      diagnosticDirectives: { directives: [[0, 1, 2, 3]] },
    },
  };
  const body = Buffer.from(JSON.stringify(message));
  const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  assert.deepEqual(decodeFrames(Buffer.concat([frame, frame])), [message, message]);
  assert.throws(() => decodeFrames(frame.subarray(0, -1)), /truncated/);
  assert.throws(
    () => decodeFrames(Buffer.from("Content-Length: 1\r\nContent-Length: 1\r\n\r\n0")),
    /ambiguous/,
  );
  assert.throws(() => decodeFrames(Buffer.concat([frame, Buffer.from("garbage")])), /truncated/);
});

void test("phase and resource receipts cannot pass with incomplete or early-failure output", () => {
  const text =
    "Content mapper request wait time: 0.392s\nCheck time: 2.478s\nTypes: 4684959\nInstantiations: 49119876\nMemory used: 2296581K\n";
  assert.deepEqual(metrics(text), {
    mapperWaitSeconds: 0.392,
    checkSeconds: 2.478,
    types: 4684959,
    instantiations: 49119876,
    checkerMemoryKiB: 2296581,
  });
  assert.throws(() => metrics(text.replace("Check time: 2.478s\n", "")), /Check time/);
  assert.deepEqual(
    resourceUsage(
      "Command exited with non-zero status 2\nwall_s=3.1\nuser_s=10.2\nsystem_s=1.0\nmax_rss_kib=123456\nstatus=2\n",
    ),
    { wall_s: 3.1, user_s: 10.2, system_s: 1, max_rss_kib: 123456, status: 2 },
  );
  assert.throws(() => resourceUsage("wall_s=0.1"), /user_s/);
  for (const token of ["..", "NaN", "Infinity", "-1", "1.2.3", "9".repeat(400)]) {
    assert.throws(() => metrics(text.replace("0.392", token)), /numeric/);
    assert.throws(
      () => resourceUsage(`wall_s=${token}\nuser_s=1\nsystem_s=0\nmax_rss_kib=10\nstatus=2`),
      /numeric/,
    );
  }
});
