import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

type LifecycleMode = "baseline" | "allocator" | "memcheck";

interface LifecycleIdentity {
  readonly label: string;
  readonly side: "public" | "source";
  readonly mode: LifecycleMode;
  readonly attempt: 1;
  readonly nativeEntry: string;
  readonly binary: string;
  readonly binarySha256: string;
  readonly token?: string;
  readonly retainedBinary: string;
  readonly viteEntry: string;
  readonly vueEntry: string;
  readonly typescriptEntry: string;
  readonly pluginEntry: string;
  readonly museaEntry: string;
  readonly fixtureParent: string;
  readonly diagnosticRuntime?: {
    readonly packagePath: string;
    readonly realpath: string;
    readonly sha256: string;
  };
  readonly loader?: { readonly path: string; readonly sha256: string };
}

interface NativeBinding {
  analyzeSfc(
    source: string,
    options: { readonly filename: string; readonly cache: boolean; readonly profile: boolean },
  ): unknown;
  getNativeBuildToken(): string;
}

interface PalettePacket {
  readonly controls: Array<{ readonly name: string; readonly required: boolean }>;
  readonly json: string;
  readonly typescript: string;
}

interface LifecyclePacket {
  readonly newline: string;
  readonly source: string;
  readonly native: unknown;
  analysis?: unknown;
  palette?: PalettePacket;
}

interface TypeScriptName {
  getText(sourceFile: TypeScriptSourceFile): string;
}

interface TypeScriptStringLiteralName extends TypeScriptName {
  readonly text: string;
}

interface TypeScriptTypeElement {
  readonly name: TypeScriptName;
  readonly questionToken?: unknown;
}

interface TypeScriptSourceFile {
  readonly parseDiagnostics: readonly unknown[];
  readonly statements: readonly unknown[];
}

interface TypeScriptRuntime {
  readonly version: string;
  readonly ScriptTarget: { readonly Latest: number };
  readonly ScriptKind: { readonly TS: number };
  createSourceFile(
    fileName: string,
    sourceText: string,
    languageVersion: number,
    setParentNodes: boolean,
    scriptKind: number,
  ): TypeScriptSourceFile;
  isTypeAliasDeclaration(node: unknown): node is { readonly type: unknown };
  isTypeLiteralNode(node: unknown): node is { readonly members: readonly TypeScriptTypeElement[] };
  isInterfaceDeclaration(
    node: unknown,
  ): node is { readonly members: readonly TypeScriptTypeElement[] };
  isStringLiteral(node: TypeScriptName): node is TypeScriptStringLiteralName;
}

const members =
  'spaced? : string;\ntabbed?\t: number;\ncommented? /* question */ : boolean;\ncompact?: string;\nrequired : number;\nreadonly read? : string;\n"scope.name"? : string;\ncafé? : number;\n"why?:"? /* trivia */ : boolean;';
export const componentSource =
  '<script setup lang="ts">const props = defineProps<{ ' +
  members +
  " }>();</script><template><div>{{ props.required }}</div></template>";
export const artSource =
  '<script setup lang="ts">defineArt("./SpacingProbe.vue", { title: "Optional Spacing" })</script><art><variant name="Default"><SpacingProbe :required="1" /></variant></art>';
const names = [
  "spaced",
  "tabbed",
  "commented",
  "compact",
  "required",
  "read",
  "scope.name",
  "café",
  "why?:",
];
const types = [
  "string",
  "number",
  "boolean",
  "string",
  "number",
  "string",
  "string",
  "number",
  "boolean",
];
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const fileHash = (file: string) => hash(fs.readFileSync(file));
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");

/** Exercise the whole native and HTTP lifecycle while retaining every completed boundary. */
export async function runLifecycleWorker(identityFile: string, output: string): Promise<void> {
  const identity = JSON.parse(fs.readFileSync(identityFile, "utf8")) as LifecycleIdentity;
  assert.equal(identity.attempt, 1);
  assert.ok(["baseline", "allocator", "memcheck"].includes(identity.mode));
  const trace = (phase: string, extra: Readonly<Record<string, unknown>> = {}) => {
    fs.appendFileSync(
      path.join(output, "phases.jsonl"),
      JSON.stringify({ phase, label: identity.label, ...extra }) + "\n",
    );
    console.log(JSON.stringify({ phase, label: identity.label, ...extra }));
  };
  process.once("beforeExit", (code) => trace("process-before-exit", { code }));
  process.once("exit", (code) => trace("process-exit", { code }));
  trace("worker-start");
  const processMaps = fs.readFileSync("/proc/self/maps", "utf8");
  const mappedFiles = processMaps
    .split("\n")
    .map((line) => line.trim().split(/\s+/).at(-1))
    .filter((file): file is string => file?.startsWith("/") === true);
  if (identity.mode === "allocator") {
    assert.ok(identity.diagnosticRuntime, "allocator runtime identity missing");
    assert.equal(
      fs.realpathSync(process.env.LD_PRELOAD ?? ""),
      identity.diagnosticRuntime.realpath,
    );
    assert.equal(fileHash(identity.diagnosticRuntime.realpath), identity.diagnosticRuntime.sha256);
    assert.ok(
      mappedFiles.some(
        (file) =>
          path.basename(file) === "libc_malloc_debug.so.0" &&
          fs.realpathSync(file) === identity.diagnosticRuntime?.realpath,
      ),
      "malloc debug image is not mapped in the worker",
    );
    trace("allocator-runtime-loaded", identity.diagnosticRuntime);
  } else if (identity.mode === "memcheck") {
    assert.ok(identity.loader, "tsx loader identity missing");
    assert.equal(fileHash(identity.loader.path), identity.loader.sha256);
    const memcheckImages = mappedFiles.filter((file) => file.includes("/vgpreload_memcheck-"));
    assert.ok(memcheckImages.length > 0, "worker process is outside Valgrind memcheck");
    trace("memcheck-runtime-loaded", { images: memcheckImages, loader: identity.loader });
  }
  const root = fs.mkdtempSync(path.join(identity.fixtureParent, "lifecycle-"));
  const component = path.join(root, "src/SpacingProbe.vue");
  const art = path.join(root, "src/SpacingProbe.art.vue");
  fs.mkdirSync(path.dirname(component), { recursive: true });
  fs.writeFileSync(component, componentSource);
  fs.writeFileSync(art, artSource);
  fs.writeFileSync(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
  const { default: ts } = (await import(pathToFileURL(identity.typescriptEntry).href)) as {
    default: TypeScriptRuntime;
  };
  assert.equal(ts.version, "6.0.3");
  const truth = ts.createSourceFile(
    "truth.ts",
    "type Props = { " + members + " }",
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  assert.deepEqual(truth.parseDiagnostics, []);
  const alias = truth.statements[0];
  assert.ok(ts.isTypeAliasDeclaration(alias) && ts.isTypeLiteralNode(alias.type));
  assert.deepEqual(
    alias.type.members.map((member) => member.questionToken !== undefined),
    names.map((name) => name !== "required"),
  );
  const require = createRequire(identity.nativeEntry);
  trace("native-load-begin");
  const binding = require(identity.nativeEntry) as NativeBinding;
  const images = () =>
    Object.keys(require.cache)
      .filter((file) => file.endsWith(".node") && path.basename(file).startsWith("vize-vitrine"))
      .map((file) => fs.realpathSync(file));
  assert.deepEqual(images(), [identity.binary]);
  assert.equal(fileHash(identity.binary), identity.binarySha256);
  assert.equal(fileHash(identity.retainedBinary), identity.binarySha256);
  const token = binding.getNativeBuildToken();
  assert.equal(typeof token, "string");
  if (identity.side === "source") assert.equal(token, identity.token);
  const packets: LifecyclePacket[] = [];
  const retain = () =>
    write(path.join(output, "packets.json"), {
      identity,
      loadedToken: token,
      root,
      fixtureSha256: hash(componentSource),
      artSha256: hash(artSource),
      throughputMeasured: false,
      packets,
    });
  retain();
  trace("native-loaded", {
    binary: identity.binary,
    retainedBinary: identity.retainedBinary,
    sha256: identity.binarySha256,
    token,
  });
  const { createServer } = await import(pathToFileURL(identity.viteEntry).href);
  const { default: vize } = await import(pathToFileURL(identity.pluginEntry).href);
  const { musea } = await import(pathToFileURL(identity.museaEntry).href);
  trace("create-server-begin");
  const server = await createServer({
    root,
    configFile: false,
    resolve: { alias: { vue: identity.vueEntry } },
    plugins: [vize(), musea({ include: ["src/**/*.art.vue"] })],
    server: { host: "127.0.0.1", port: 0 },
  });
  trace("create-server-end");
  let failure: unknown;
  try {
    trace("listen-begin");
    await server.listen();
    trace("listen-end");
    const address = server.httpServer.address();
    assert.ok(address && typeof address !== "string");
    const endpoint =
      "http://127.0.0.1:" + address.port + "/__musea__/api/arts/" + encodeURIComponent(art);
    for (const newline of ["\n", "\r\n"]) {
      const source = componentSource.replaceAll("\n", newline);
      trace("write-begin", { newline });
      await fs.promises.writeFile(component, source);
      trace("write-end", { newline });
      trace("native-begin", { newline });
      const native = binding.analyzeSfc(source, {
        filename: component,
        cache: false,
        profile: false,
      });
      const expected = {
        emits: [],
        props: names.map((name, i) => ({
          name,
          type: types[i],
          required: name === "required" || (identity.side === "public" && i < 3),
        })),
      };
      assert.deepEqual(
        native,
        expected,
        "published optional flags differ only for the three proven cases",
      );
      trace("native-end", { newline });
      const packet: LifecyclePacket = { newline, source, native };
      packets.push(packet);
      retain();
      trace("http-analysis-begin", { newline });
      const analysis = await fetch(endpoint + "/analysis");
      assert.equal(analysis.status, 200);
      packet.analysis = (await analysis.json()) as unknown;
      assert.deepEqual(packet.analysis, native);
      retain();
      trace("http-analysis-end", { newline });
      trace("http-palette-begin", { newline });
      const palette = await fetch(endpoint + "/palette");
      assert.equal(palette.status, 200);
      packet.palette = (await palette.json()) as PalettePacket;
      assert.deepEqual(
        packet.palette.controls.map(({ name, required }) => ({ name, required })),
        expected.props.map(({ name, required }) => ({ name, required })),
      );
      assert.deepEqual(JSON.parse(packet.palette.json).controls, packet.palette.controls);
      const exported = ts.createSourceFile(
        "palette.ts",
        packet.palette.typescript,
        ts.ScriptTarget.Latest,
        true,
        ts.ScriptKind.TS,
      );
      assert.deepEqual(exported.parseDiagnostics, []);
      const declaration = exported.statements[0];
      assert.ok(ts.isInterfaceDeclaration(declaration));
      assert.deepEqual(
        declaration.members.map((member) => ({
          name: ts.isStringLiteral(member.name) ? member.name.text : member.name.getText(exported),
          required: member.questionToken === undefined,
        })),
        expected.props.map(({ name, required }) => ({ name, required })),
      );
      retain();
      trace("http-palette-end", { newline });
    }
  } catch (error: unknown) {
    failure = error;
    trace("functional-error", {
      message: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : undefined,
    });
  } finally {
    retain();
    trace("close-begin");
    await server.close();
    trace("close-end");
  }
  assert.deepEqual(images(), [identity.binary]);
  assert.equal(fileHash(identity.binary), identity.binarySha256);
  assert.equal(fileHash(identity.retainedBinary), identity.binarySha256);
  trace("worker-complete");
  if (failure) throw failure;
}
