import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { prepareExperimentalRouterProject } from "./support/router-experimental-provider.ts";
import {
  normalizeExperimentalRouterPackets,
  type Json,
} from "./support/router-experimental-transport.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { resolveTypecheckRuntime } from "./support/typecheck-dependency.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const page = "src/pages/products/[id=int].vue";
const output = path.join(root, "target/differential/experimental-router-variable-roots");
const fixture = path.join(
  root,
  "tests/_fixtures/differential/lsp/experimental-router-params-10518",
);
const options = {
  completion: true,
  editor: true,
  ecosystem: true,
  hover: true,
  lint: false,
  typecheck: true,
};
const methods = [
  "textDocument/hover",
  "textDocument/completion",
  "textDocument/definition",
  "textDocument/references",
];
function position(source: string, offset: number) {
  assert.ok(offset >= 0);
  const prefix = source.slice(0, offset);
  return {
    line: prefix.split("\n").length - 1,
    character: prefix.length - prefix.lastIndexOf("\n") - 1,
  };
}
function range(source: string, offset: number, length: number) {
  return { start: position(source, offset), end: position(source, offset + length) };
}
function relocateExpected(value: Json, offset: number, revision: number): Json {
  if (Array.isArray(value)) return value.map((entry) => relocateExpected(entry, offset, revision));
  if (value !== null && typeof value === "object") {
    const result = Object.fromEntries(
      Object.entries(value).map(([key, entry]) => [
        key,
        key === "revision" ? revision : relocateExpected(entry, offset, revision),
      ]),
    );
    if (
      typeof value.fileName === "string" &&
      typeof value.name === "string" &&
      typeof value.position === "number"
    )
      result.position = offset;
    return result;
  }
  return value;
}

// These roots deliberately have different lengths. Their private source bytes
// are retained, not claimed equivalent to the separate fixed-length golden.
test(
  "experimental route packets preserve authored ranges under short and long quoted Unicode roots",
  { timeout: 600_000 },
  async () => {
    const runtime = resolveTypecheckRuntime(root);
    assert.ok(runtime, "the actual native TypeScript runtime is required");
    fs.mkdirSync(output, { recursive: true });
    const provider = await prepareExperimentalRouterProject(path.join(output, "provider"));
    const ts = createRequire(path.join(provider.root, "package.json"))(
      "typescript",
    ) as typeof import("typescript");
    const expected = JSON.parse(fs.readFileSync(path.join(fixture, "lf.expected.json"), "utf8"));
    const owner = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-router-variable-")));
    const receipts: unknown[] = [];
    const failures: string[] = [];
    const compare = (actual: unknown, authority: unknown) => {
      try {
        assert.deepEqual(actual, authority);
      } catch (error) {
        if (!(error instanceof assert.AssertionError)) throw error;
        failures.push(error.message);
      }
    };
    try {
      for (const label of ["short-'雪😀", `long-${"x".repeat(100)}-"雪😀`]) {
        const directory = path.join(owner, label);
        assert.equal(path.dirname(directory), owner);
        fs.mkdirSync(directory);
        fs.cpSync(provider.root, directory, {
          recursive: true,
          filter: (file) => !file.split(path.sep).includes("node_modules"),
        });
        fs.symlinkSync(
          path.join(provider.root, "node_modules"),
          path.join(directory, "node_modules"),
          "junction",
        );
        const source = expected.source.replace(
          "const route =",
          `const workspaceLabel = ${JSON.stringify(`${directory}/keep-root-literal`)}\nconst route =`,
        );
        fs.writeFileSync(path.join(directory, page), source);
        writeVueConfigFixture(directory, { lsp: options, typeChecker: { corsaPath: runtime } });
        const uri = pathToFileURL(path.join(directory, page)).href;
        const session = new LspSession();
        const raw: unknown[] = [];
        session.responseObservers.push((packet) => raw.push(packet));
        session.notificationObservers.push((method, params) => raw.push({ method, params }));
        const receipt = { directory, source, raw, stages: [] as unknown[], stderr: "" };
        receipts.push(receipt);
        try {
          await session.initialize(directory, options);
          session.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version: 1, text: source },
          });
          for (const [version, text] of [
            [1, source],
            [2, source.replace("route.params.page.toFixed()", "route.params.page.toUpperCase()")],
            [3, source],
          ] as const) {
            if (version !== 1)
              session.notify("textDocument/didChange", {
                textDocument: { uri, version },
                contentChanges: [{ text }],
              });
            const published = await session.waitForNotification(
              "textDocument/publishDiagnostics",
              (value) => {
                const packet = value as { uri: string; version: number };
                return packet.uri === uri && packet.version === version;
              },
            );
            const diagnostics =
              version === 2
                ? expected.invalidDiagnostics.diagnostics.map((item: Record<string, Json>) => ({
                    ...item,
                    range: range(text, text.indexOf("toUpperCase"), "toUpperCase".length),
                  }))
                : [];
            compare(published, { uri, version, diagnostics });
            const requests = [];
            const resolved = [];
            for (const method of methods) {
              const params = {
                textDocument: { uri },
                position: position(
                  text,
                  text.indexOf("route.params.page") +
                    (method === "textDocument/completion" ? 13 : 14),
                ),
                ...(method === "textDocument/references"
                  ? { context: { includeDeclaration: true } }
                  : {}),
              };
              const started = performance.now();
              const response = await session.request(method, params);
              requests.push({ method, params, response, elapsedMs: performance.now() - started });
              if (method === "textDocument/completion") {
                assert.ok(Array.isArray(response));
                for (const item of response)
                  resolved.push(await session.request("completionItem/resolve", item));
              }
            }
            const completion = requests[1].response as Array<{
              data?: { vizeCompletion?: { item?: { data?: { fileName?: string } } } };
            }>;
            const file = completion[0]?.data?.vizeCompletion?.item?.data?.fileName;
            assert.ok(file, "real native completion data must identify its generated document");
            const bytes = fs.readFileSync(file);
            const generated = {
              file,
              source: bytes.toString("utf8"),
              sha256: createHash("sha256").update(bytes).digest("hex"),
            };
            const stage = {
              version,
              published,
              requests,
              resolved,
              generated,
              independentPosition: null as number | null,
              parsedReferences: [] as readonly unknown[],
            };
            receipt.stages.push(stage);
            const tree = ts.createSourceFile(
              file,
              generated.source,
              ts.ScriptTarget.Latest,
              true,
              ts.ScriptKind.TS,
            );
            stage.parsedReferences = tree.referencedFiles;
            const resolvedReferences = tree.referencedFiles.map((reference) =>
              ts.resolveTripleslashReference(reference.fileName, file),
            );
            assert.equal(
              resolvedReferences.length,
              1,
              "the independent parser must find the actual provider declaration",
            );
            compare(fs.readFileSync(resolvedReferences[0], "utf8"), provider.generatedDeclaration);
            const offsets: number[] = [];
            const labels: string[] = [];
            const visit = (node: import("typescript").Node) => {
              if (ts.isVariableDeclaration(node) && ts.isIdentifier(node.name)) {
                if (
                  node.name.text === "page" &&
                  node.initializer &&
                  ts.isPropertyAccessExpression(node.initializer) &&
                  node.initializer.getText(tree) === "route.params.page"
                )
                  offsets.push(
                    Buffer.byteLength(
                      generated.source.slice(0, node.initializer.name.getStart(tree)),
                    ),
                  );
                if (
                  node.name.text === "workspaceLabel" &&
                  node.initializer &&
                  ts.isStringLiteral(node.initializer)
                )
                  labels.push(node.initializer.text);
              }
              ts.forEachChild(node, visit);
            };
            visit(tree);
            assert.equal(
              offsets.length,
              1,
              "the independently parsed authored query declaration must be unique",
            );
            compare(labels, [`${directory}/keep-root-literal`]);
            const authority = structuredClone(expected.requests);
            const authored = [...text.matchAll(/route\.params\.page/gu)].map((match) =>
              range(text, match.index + 13, 4),
            );
            assert.equal(authored.length, 2);
            for (let index = 0; index < methods.length; index++)
              authority[index].params.position = position(
                text,
                text.indexOf("route.params.page") + (index === 1 ? 13 : 14),
              );
            authority[0].response.range = authored[0];
            const declaration = authority[3].response.find(
              (location: { uri: string }) => location.uri === "file://$PROJECT/typed-router.d.ts",
            );
            assert.ok(declaration);
            const sourceReferences = authored.map((span) => ({
              uri: "file://$PROJECT/src/pages/products/%5Bid=int%5D.vue",
              range: span,
            }));
            // GoTS escapes apostrophes in declaration URIs; the editor's source URI
            // retains them. The public control pins that ordered wire distinction.
            authority[3].response = label.startsWith("short-")
              ? [declaration, ...sourceReferences]
              : [...sourceReferences, declaration];
            const packets = requests.map(({ elapsedMs: _, ...packet }) => packet);
            // The independent public TS parser owns the private offset oracle;
            // no generated code or map comment is normalized for this family.
            compare(
              normalizeExperimentalRouterPackets(packets, [directory], directory, generated, false),
              relocateExpected(authority, offsets[0], version),
            );
            compare(
              normalizeExperimentalRouterPackets(
                resolved,
                [directory],
                directory,
                generated,
                false,
              ),
              relocateExpected(expected.resolved, offsets[0], version),
            );
            stage.independentPosition = offsets[0];
          }
        } finally {
          await session.shutdown();
          receipt.stderr = session.stderrText;
          fs.writeFileSync(
            path.join(output, "observation.json"),
            `${JSON.stringify({ dependencies: provider.dependencies, failures, receipts }, null, 2)}\n`,
          );
        }
      }
    } finally {
      provider.cleanup();
      fs.rmSync(owner, { recursive: true, force: true });
    }
    assert.deepEqual(
      failures,
      [],
      "every variable-root authored packet must match its independent oracle",
    );
  },
);
