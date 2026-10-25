import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  experimentalRouterFixture,
  experimentalRouterPage,
  prepareExperimentalRouterProject,
} from "./support/router-experimental-provider.ts";

type Packet = {
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
};
type Span = [number, number, number, number, number, number];
type Transform = { text: string; extension: string; mappings: Span[]; diagnostics: unknown[] };
type Response = {
  jsonrpc: string;
  id: number;
  error?: unknown;
  result: Partial<Transform> & { configIdentity?: string };
};
const mapperCorpus = new URL(
  "../_fixtures/differential/typechecker/content-mapper-experimental-router-10526/",
  import.meta.url,
);
const validPacket: Packet = { status: 0, signal: null, error: null, stdout: "", stderr: "" };
const broadParams =
  "Record<never, never> | { at: Date | undefined; heading: string | undefined; id: number; page: number; tags: string[] | undefined; }";
const accesses = [
  ["id", 11, 33],
  ["page", 12, 35],
  ["tags", 13, 49],
  ["at", 14, 43],
  ["heading", 15, 50],
  ["id", 17, 32],
  ["page", 17, 64],
] as const;
function disabledPacket(lineOffset: number): Packet {
  return {
    ...validPacket,
    status: 2,
    stdout: accesses
      .map(
        ([name, line, column]) =>
          `${experimentalRouterPage}(${line + lineOffset},${column}): error TS2339: Property '${name}' does not exist on type '${broadParams}'.\n  Property '${name}' does not exist on type 'Record<never, never>'.\n`,
      )
      .join(""),
  };
}
function invalidPacket(lineOffset: number): Packet {
  return {
    ...validPacket,
    status: 2,
    stdout: `${experimentalRouterPage}(${12 + lineOffset},7): error TS2322: Type 'number' is not assignable to type 'string'.\n`,
  };
}
function frame(value: unknown): Buffer {
  const body = Buffer.from(JSON.stringify(value));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
}
function decodeFrames(bytes: Buffer): Response[] {
  const responses: Response[] = [];
  while (bytes.length) {
    const end = bytes.indexOf("\r\n\r\n");
    assert.notEqual(end, -1, "complete frame header");
    const match = /^Content-Length: (\d+)$/u.exec(bytes.subarray(0, end).toString());
    assert.ok(match);
    const length = Number(match[1]),
      start = end + 4;
    assert.ok(bytes.length >= start + length, "complete frame body");
    responses.push(JSON.parse(bytes.subarray(start, start + length).toString()) as Response);
    bytes = bytes.subarray(start + length);
  }
  return responses;
}
function assertRouteMappings(source: string, result: Transform) {
  assert.equal(result.extension, ".tsx");
  assert.deepEqual(result.diagnostics, []);
  const authored = Buffer.from(source),
    generated = Buffer.from(result.text);
  const snippets = [
    "const id: number = route.params.id",
    "const page: number = route.params.page",
    "const tags: string[] | undefined = route.params.tags",
    "const at: Date | undefined = route.params.at",
    "const heading: string | undefined = route.params.heading",
    "route.params.id.toFixed()",
    "route.params.page.toFixed()",
  ];
  const expected = snippets.map((snippet) => {
    const bytes = Buffer.from(snippet),
      originalStart = authored.indexOf(bytes),
      generatedStart = generated.indexOf(bytes);
    assert.notEqual(originalStart, -1, snippet);
    assert.notEqual(generatedStart, -1, snippet);
    return [generatedStart, bytes.length, originalStart, bytes.length, 0, 1_048_575];
  });
  const originalStarts = new Set(expected.map((span) => span[2]));
  assert.deepEqual(
    result.mappings.filter((span) => originalStarts.has(span[2])),
    expected,
    "all seven route accesses retain exact UTF-8 protocol spans",
  );
}

test("mapper packet oracle fixes the complete negative vector and frame byte lengths", () => {
  assert.equal(disabledPacket(0).stdout.match(/error TS2339:/gu)!.length, 7);
  assert.equal(
    invalidPacket(1).stdout,
    `${experimentalRouterPage}(13,7): error TS2322: Type 'number' is not assignable to type 'string'.\n`,
  );
  const packet = { text: "café 😀 ' \\\"\r\n", mappings: [] };
  assert.deepEqual(decodeFrames(frame(packet)), [packet]);
  assert.throws(() => decodeFrames(frame(packet).subarray(0, -1)));
});

test("the official mapper host is pinned independently of the provider's reference compiler", () => {
  const packageManifest = JSON.parse(
    fs.readFileSync(
      new URL(
        "../fixtures/typechecker/experimental-router-mapper-host/package.json.txt",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const lock = JSON.parse(
    fs.readFileSync(
      new URL(
        "../fixtures/typechecker/experimental-router-mapper-host/package-lock.json.txt",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  assert.deepEqual(packageManifest.dependencies, { typescript: "7.1.0-dev.20261017.1" });
  assert.deepEqual(lock.packages[""].dependencies, packageManifest.dependencies);
  assert.equal(
    lock.packages["node_modules/typescript"].integrity,
    "sha512-d8K6oDCprwXc9c2A9Miu9J+JeyN7kK+QGijoyGS9uA1KKzMoGer9+5df77k/mFQ5y7TQJQ8quWDsKkSCvcdIjw==",
  );
  assert.equal(Object.keys(lock.packages).length, 9);
  for (const [name, entry] of Object.entries(lock.packages) as [
    string,
    { version: string; resolved?: string; integrity?: string },
  ][]) {
    if (!name) continue;
    assert.equal(entry.version, "7.1.0-dev.20261017.1", name);
    assert.ok(entry.resolved?.startsWith("https://registry.npmjs.org/"), name);
    assert.match(entry.integrity!, /^sha512-[A-Za-z0-9+/]+={0,2}$/u, name);
  }
});

const binary = process.env.VIZE_TEST_BIN;
const host = process.env.VIZE_TEST_CONTENT_MAPPER_TSGO;
const required = process.env.VIZE_TEST_ROUTER_EXPERIMENTAL_REQUIRED === "1";
test(
  "published experimental Router content-mapper project context has exact provider diagnostics and mappings",
  { skip: !required && (!binary || !host), timeout: 240_000 },
  async () => {
    assert.ok(
      binary && path.isAbsolute(binary) && fs.statSync(binary).isFile(),
      "absolute source-built VIZE_TEST_BIN",
    );
    assert.ok(
      host && path.isAbsolute(host) && fs.statSync(host).isFile(),
      "absolute official mapper host VIZE_TEST_CONTENT_MAPPER_TSGO",
    );
    const outputRoot =
      process.env.VIZE_TEST_ROUTER_OUTPUT ??
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-router-mapper-oracle-"));
    fs.mkdirSync(outputRoot, { recursive: true });
    const processes: unknown[] = [];
    const run = (
      label: string,
      executable: string,
      args: string[],
      cwd: string,
      input?: Buffer,
    ): Packet => {
      const started = performance.now();
      const raw = spawnSync(executable, args, {
        cwd,
        input,
        encoding: "utf8",
        timeout: 90_000,
        maxBuffer: 32 * 1024 * 1024,
      });
      const packet = {
        status: raw.status,
        signal: raw.signal,
        error: raw.error?.message ?? null,
        stdout: raw.stdout ?? "",
        stderr: raw.stderr ?? "",
      };
      processes.push({
        label,
        executable,
        args,
        elapsedMs: performance.now() - started,
        ...packet,
      });
      fs.writeFileSync(
        path.join(outputRoot, "mapper-processes.json"),
        `${JSON.stringify(processes, null, 2)}\n`,
      );
      return packet;
    };
    const project = await prepareExperimentalRouterProject(outputRoot);
    try {
      fs.writeFileSync(
        path.join(outputRoot, "source-cli-identity.json"),
        `${JSON.stringify({ binary, sha256: createHash("sha256").update(fs.readFileSync(binary)).digest("hex"), host }, null, 2)}\n`,
      );
      assert.deepEqual(run("official-host-version", host, ["--version"], project.root), {
        ...validPacket,
        stdout: "Version 7.1.0-dev.20261017.1\n",
      });
      assert.equal(
        fs.readFileSync(new URL("Page.valid.vue.txt", mapperCorpus), "utf8"),
        project.pageSource,
      );
      assert.equal(
        fs.readFileSync(new URL("Page.invalid.vue.txt", mapperCorpus), "utf8"),
        project.pageSource.replace("const page: number", "const page: string"),
      );
      assert.equal(
        fs.readFileSync(new URL("typed-router.d.ts.txt", mapperCorpus), "utf8"),
        project.generatedDeclaration,
      );
      assert.deepEqual(
        fs.readFileSync(new URL("provider.json.txt", mapperCorpus)),
        fs.readFileSync(path.join(experimentalRouterFixture, "provider.json")),
      );
      assert.deepEqual(
        fs.readFileSync(new URL("mapper-host-package-lock.json.txt", mapperCorpus)),
        fs.readFileSync(
          new URL(
            "../fixtures/typechecker/experimental-router-mapper-host/package-lock.json.txt",
            import.meta.url,
          ),
        ),
      );
      const packageRoot = path.join(project.root, "node_modules/router-source-mapper");
      fs.mkdirSync(packageRoot, { recursive: true });
      fs.writeFileSync(
        path.join(packageRoot, "package.json"),
        JSON.stringify({
          name: "router-source-mapper",
          private: true,
          typescript: {
            contentMapper: {
              exec: [binary, "content-mapper"],
              compilerOptions: ["checkJs", "noUnusedLocals", "rootDir"],
              dynamicConfig: true,
            },
          },
        }),
      );
      fs.copyFileSync(
        new URL("tsconfig.mapper.json.txt", mapperCorpus),
        path.join(project.root, "tsconfig.mapper.json"),
      );
      fs.writeFileSync(
        path.join(project.root, "tsconfig.disabled.json"),
        JSON.stringify({
          extends: "./tsconfig.mapper.json",
          vueCompilerOptions: { plugins: [] },
        }),
      );
      const config = path.join(project.root, "tsconfig.mapper.json"),
        disabledConfig = path.join(project.root, "tsconfig.disabled.json");
      for (const newline of ["\n", "\r\n"]) {
        // A multibyte prefix makes every subsequent authored mapping sensitive to UTF-8 offsets.
        const source = project.pageSource
          .replace(/\r?\n/gu, "\n")
          .replace("\n", "\n// café 😀 ' \\\"\n")
          .replace(/\n/gu, newline);
        fs.writeFileSync(project.pagePath, source);
        const label = newline === "\n" ? "lf" : "crlf";
        const request = (id: number, method: string, params: unknown) => ({
          jsonrpc: "2.0",
          id,
          method,
          params,
        });
        const open = (id: number, handle: string, file: string) =>
          request(id, "openProject", {
            configFileName: file,
            projectHandle: handle,
            compilerOptions: { rootDir: project.root },
          });
        const transform = (id: number, handle: string) =>
          request(id, "transform", {
            fileName: project.pagePath,
            projectHandle: handle,
            content: source,
          });
        const wire = run(
          `${label}-same-process-two-projects`,
          binary,
          ["content-mapper"],
          project.root,
          Buffer.concat([
            frame(request(1, "initialize", { positionEncodings: ["utf-8"] })),
            frame(open(2, "on", config)),
            frame(open(3, "off", disabledConfig)),
            frame(transform(4, "on")),
            frame(transform(5, "off")),
            frame(transform(6, "on")),
          ]),
        );
        assert.deepEqual({ ...wire, stdout: "" }, validPacket);
        const packets = decodeFrames(Buffer.from(wire.stdout));
        fs.writeFileSync(
          path.join(outputRoot, `${label}-complete-mapper-packets.json`),
          `${JSON.stringify(packets, null, 2)}\n`,
        );
        assert.equal(packets.length, 6);
        assert.deepEqual(
          packets.map((packet) => [packet.jsonrpc, packet.id, packet.error ?? null]),
          [1, 2, 3, 4, 5, 6].map((id) => ["2.0", id, null]),
        );
        const enabled = packets[3].result as Transform,
          disabled = packets[4].result as Transform;
        assert.deepEqual(
          packets[5].result,
          enabled,
          "one process retains each project's authority",
        );
        assert.notEqual(packets[1].result.configIdentity, packets[2].result.configIdentity);
        const routeType = `<import('vue-router/auto-routes')._RouteNamesForFilePath<${JSON.stringify(experimentalRouterPage)}>>`;
        assert.equal(enabled.text.split(`useRoute${routeType}()`).length - 1, 1);
        assert.equal(disabled.text.split(`useRoute${routeType}()`).length - 1, 0);
        assertRouteMappings(source, enabled);
        assertRouteMappings(source, disabled);
        const check = (name: string, configName: string, extra: string[] = []) =>
          run(
            `${label}-${name}`,
            host,
            ["--runExternalCode", "--noEmit", "-p", configName, "--pretty", "false", ...extra],
            project.root,
          );
        const reference = (name: string) =>
          run(
            `${label}-reference-${name}`,
            process.execPath,
            [
              path.join(project.root, "node_modules/vue-tsc/bin/vue-tsc.js"),
              "--noEmit",
              "-p",
              "tsconfig.json",
              "--pretty",
              "false",
            ],
            project.root,
          );
        assert.deepEqual(check("valid", "tsconfig.mapper.json"), validPacket);
        assert.deepEqual(reference("valid"), validPacket);
        assert.deepEqual(check("disabled", "tsconfig.disabled.json"), disabledPacket(1));
        fs.writeFileSync(
          project.pagePath,
          source.replace("const page: number", "const page: string"),
        );
        assert.deepEqual(check("invalid", "tsconfig.mapper.json"), invalidPacket(1));
        assert.deepEqual(reference("invalid"), invalidPacket(1));
        fs.writeFileSync(project.pagePath, source);
        assert.deepEqual(check("repaired", "tsconfig.mapper.json"), validPacket);
        assert.deepEqual(reference("repaired"), validPacket);
        const rootConfig = path.join(project.root, "tsconfig.root.json");
        const rootOptions = {
          extends: "./tsconfig.json",
          compilerOptions: { rootDir: "./src" },
          contentMappers: [{ package: "router-source-mapper", extensions: [".vue"] }],
        };
        fs.writeFileSync(rootConfig, JSON.stringify(rootOptions));
        assert.deepEqual(check("authored-root-mismatch", "tsconfig.root.json"), disabledPacket(1));
        assert.deepEqual(
          check("host-root-override", "tsconfig.root.json", ["--rootDir", project.root]),
          validPacket,
        );
        fs.writeFileSync(
          rootConfig,
          JSON.stringify({
            ...rootOptions,
            vueCompilerOptions: {
              plugins: [{ name: "vue-router/volar/sfc-typed-router", options: { rootDir: "." } }],
            },
          }),
        );
        assert.deepEqual(
          check("explicit-plugin-root-precedence", "tsconfig.root.json"),
          validPacket,
        );
      }
    } finally {
      project.cleanup();
    }
  },
);
