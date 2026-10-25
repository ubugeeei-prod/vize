import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { FRESH_EDITOR_CAPABILITIES } from "./support/lsp/fresh-capability-policy.ts";
import { withFreshVueProject } from "./support/lsp/fresh-vue-project.ts";
import {
  IntrinsicHoverTimings,
  intrinsicWorkspaceControls,
} from "./support/lsp/intrinsic-hover-observation.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { JsonRpcMessage } from "./support/lsp/protocol.ts";
import { LspRequestError } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

const references = JSON.parse(
  readPinnedArtifact(
    path.join(root, "tests/_fixtures/differential/lsp/native-html-tag-range-10449"),
    {
      path: "responses.expected.json",
      sha256: "cf0216381749032dae6a7d1903a57793aa30988af0896d4dfda8027366873220",
    },
  ).toString("utf8"),
) as Array<{ result: { contents: unknown } }>;
const sourceText =
  'import Counter from "./Counter.vue";\nexport const text = "<button>";\nexport const comparison = 1 < 2;\nexport const view = <div data-note="🧭"><button disabled title="Save">Save</button><input type="text"/><Counter count={1}></Counter></div>;\n';

test("JSX intrinsic hovers share complete DOM packets and preserve resolved project gates", async (t) => {
  const runtime = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript runtime for native JSX DOM hover",
    "TypeScript runtime unavailable",
  );
  if (!runtime) return;
  const previous = process.env.CORSA_PATH;
  process.env.CORSA_PATH = runtime;
  try {
    for (const mode of ["none", "omitted", "on", "off"] as const) {
      await t.test(mode, async (caseTest) => {
        await withFreshVueProject(caseTest, `intrinsic-${mode}`, async (session, workspace) => {
          const enabled = mode !== "off";
          if (mode !== "none") {
            const typeChecker: Record<string, unknown> = { corsaPath: runtime };
            if (mode !== "omitted") typeChecker.jsxTypecheck = enabled;
            // Use the actual canonical API: fixture adapters deliberately inject
            // an explicit opt-out and cannot qualify project default behavior.
            fs.writeFileSync(
              path.join(workspace, "vite.config.mjs"),
              `export default ${JSON.stringify({ vue: { languageServer: { hover: true, lint: false, typecheck: true }, typeChecker } })};\n`,
            );
          }
          const packets: JsonRpcMessage[] = [];
          session.responseObservers.push((packet) => packets.push(packet));
          const initialize = (await session.initialize(workspace, {
            editor: true,
            hover: true,
            lint: false,
            typecheck: true,
          })) as { capabilities: unknown };
          assert.deepEqual(initialize.capabilities, {
            ...FRESH_EDITOR_CAPABILITIES,
            experimental: { vize: { jsxTypecheck: enabled } },
          });
          let unavailableId = 0;
          await assert.rejects(
            session.request("vize/nativeReferences", {
              textDocument: { uri: pathToFileURL(path.join(workspace, "src/Counter.vue")).href },
              position: { line: 0, character: 0 },
              context: { includeDeclaration: true },
            }),
            (error) => {
              assert.ok(error instanceof LspRequestError);
              unavailableId = error.id;
              assert.deepEqual(
                {
                  method: error.method,
                  code: error.code,
                  data: error.data,
                  message: error.message,
                },
                {
                  method: "vize/nativeReferences",
                  code: -32601,
                  data: "vize/nativeReferences",
                  message: "vize/nativeReferences: Method not found",
                },
              );
              return true;
            },
          );
          assert.deepEqual(
            packets.find((packet) => packet.id === unavailableId),
            {
              jsonrpc: "2.0",
              id: unavailableId,
              error: {
                code: -32601,
                message: "Method not found",
                data: "vize/nativeReferences",
              },
            },
          );
          const observations: unknown[] = [
            {
              initialize,
              workspaceControls: intrinsicWorkspaceControls(workspace),
            },
          ];
          const timings = new IntrinsicHoverTimings();
          let complete = false;
          try {
            for (const [extension, languageId] of [
              ["jsx", "javascriptreact"],
              ["tsx", "typescriptreact"],
            ]) {
              for (const newline of ["\n", "\r\n"]) {
                const label = `${mode} ${extension} ${newline === "\n" ? "LF" : "CRLF"}`;
                const source = sourceText.replaceAll("\n", newline);
                const file = path.join(
                  workspace,
                  "src",
                  `日本語-${extension}-${newline.length}.${extension}`,
                );
                const uri = pathToFileURL(file).href;
                fs.writeFileSync(file, source);
                session.notify("textDocument/didOpen", {
                  textDocument: { uri, languageId, version: 1, text: source },
                });
                const diagnostics = await session.waitForNotification(
                  "textDocument/publishDiagnostics",
                  (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
                );
                assert.deepEqual(
                  diagnostics,
                  { uri, version: 1, diagnostics: [] },
                  `${label} complete diagnostics`,
                );
                const hover = async (text: string, at: number, expected: unknown, kind: string) => {
                  const params = { textDocument: { uri }, position: offsetToPosition(text, at) };
                  const started = performance.now();
                  const result = await session.request("textDocument/hover", params);
                  const elapsedMs = performance.now() - started;
                  timings.record(label, kind, elapsedMs);
                  observations.push({
                    label,
                    kind,
                    params,
                    result,
                    elapsedMs,
                  });
                  assert.deepEqual(result, expected, `${label} ${kind}`);
                };
                const tags = [
                  {
                    name: "button",
                    at: source.lastIndexOf("<button") + 1,
                    contents: references[0].result.contents,
                  },
                  {
                    name: "button",
                    at: source.indexOf("</button") + 2,
                    contents: references[0].result.contents,
                  },
                  {
                    name: "input",
                    at: source.indexOf("<input") + 1,
                    contents: references[1].result.contents,
                  },
                ];
                for (const tag of tags) {
                  const expected = enabled
                    ? { contents: tag.contents, range: rangeAt(source, tag.at, tag.name.length) }
                    : null;
                  for (const at of [tag.at, tag.at + tag.name.length - 1])
                    await hover(source, at, expected, `${tag.name} exact full packet`);
                }
                for (const [kind, at] of [
                  ["quoted tag", source.indexOf("<button") + 1],
                  ["comparison operator", source.indexOf("1 < 2") + 2],
                  ["attribute", source.indexOf("disabled title")],
                  ["component closing name", source.indexOf("</Counter") + 2],
                  ...tags.map((tag) => [
                    `${tag.name} exclusive endpoint`,
                    tag.at + tag.name.length,
                  ]),
                ] as Array<[string, number]>)
                  await hover(source, at, null, kind);
                const primitive = source.indexOf("comparison =");
                const primitiveHover = enabled
                  ? {
                      contents: {
                        kind: "markdown",
                        value: "```typescript\nconst comparison: boolean\n```",
                      },
                      range: rangeAt(source, primitive, 10),
                    }
                  : null;
                await hover(source, primitive, primitiveHover, "complete primitive control");
                if (mode === "on") {
                  for (let iteration = 0; iteration < 8; iteration++) {
                    for (const tag of tags)
                      await hover(
                        source,
                        tag.at,
                        { contents: tag.contents, range: rangeAt(source, tag.at, tag.name.length) },
                        `warm ${iteration} ${tag.name}`,
                      );
                    await hover(source, primitive, primitiveHover, `warm ${iteration} primitive`);
                  }
                  const changed = `// 🧭 edit${newline}${source}`;
                  session.notify("textDocument/didChange", {
                    textDocument: { uri, version: 2 },
                    contentChanges: [{ text: changed }],
                  });
                  const updated = await session.waitForNotification(
                    "textDocument/publishDiagnostics",
                    (params) => isDiagnosticsForUri(params, uri) && params.version === 2,
                  );
                  assert.deepEqual(
                    updated,
                    { uri, version: 2, diagnostics: [] },
                    `${label} changed complete diagnostics`,
                  );
                  const at = changed.lastIndexOf("<button") + 1;
                  await hover(
                    changed,
                    at,
                    { contents: references[0].result.contents, range: rangeAt(changed, at, 6) },
                    "new revision physical range",
                  );
                }
                session.notify("textDocument/didClose", { textDocument: { uri } });
              }
            }
            const refused = "export const Child = () => null;\nexport const view = <button/>;\n";
            const file = path.join(workspace, "src/Refused.tsx");
            const uri = pathToFileURL(file).href;
            fs.writeFileSync(file, refused);
            session.notify("textDocument/didOpen", {
              textDocument: { uri, languageId: "typescriptreact", version: 1, text: refused },
            });
            const refusedDiagnostics = await session.waitForNotification(
              "textDocument/publishDiagnostics",
              (params) => isDiagnosticsForUri(params, uri) && params.version === 1,
            );
            assert.deepEqual(
              refusedDiagnostics,
              { uri, version: 1, diagnostics: [] },
              `${mode} refused-source complete diagnostics`,
            );
            assert.deepEqual(
              await session.request("textDocument/hover", {
                textDocument: { uri },
                position: offsetToPosition(refused, refused.indexOf("<button") + 1),
              }),
              null,
              `${mode} incomplete native File cannot publish intrinsic facts`,
            );
            const wrongProfile = "export const view = <button/>;\n";
            const wrongFile = path.join(workspace, "src/WrongProfile.tsx");
            const wrongUri = pathToFileURL(wrongFile).href;
            fs.writeFileSync(wrongFile, wrongProfile);
            session.notify("textDocument/didOpen", {
              textDocument: {
                uri: wrongUri,
                languageId: "typescript",
                version: 1,
                text: wrongProfile,
              },
            });
            const wrongDiagnostics = await session.waitForNotification(
              "textDocument/publishDiagnostics",
              (params) => isDiagnosticsForUri(params, wrongUri) && params.version === 1,
            );
            assert.deepEqual(
              wrongDiagnostics,
              { uri: wrongUri, version: 1, diagnostics: [] },
              `${mode} non-JSX role complete diagnostics`,
            );
            assert.deepEqual(
              await session.request("textDocument/hover", {
                textDocument: { uri: wrongUri },
                position: offsetToPosition(wrongProfile, wrongProfile.indexOf("<button") + 1),
              }),
              null,
              `${mode} filename cannot grant a JSX profile`,
            );
            complete = true;
          } finally {
            fs.mkdirSync(path.join(testOutputRoot, "jsx-intrinsic-hover"), { recursive: true });
            fs.writeFileSync(
              path.join(testOutputRoot, "jsx-intrinsic-hover", `${mode}.json`),
              JSON.stringify(observations, null, 2) + "\n",
            );
            const summary = { mode, complete, requestLatency: timings.summary() };
            fs.writeFileSync(
              path.join(testOutputRoot, "jsx-intrinsic-hover", `${mode}.timings.json`),
              JSON.stringify(summary, null, 2) + "\n",
            );
            caseTest.diagnostic(
              `JSX intrinsic hover observed CI latency: ${JSON.stringify(summary)}`,
            );
          }
        });
      });
    }
  } finally {
    if (previous === undefined) delete process.env.CORSA_PATH;
    else process.env.CORSA_PATH = previous;
  }
});

function rangeAt(source: string, at: number, length: number) {
  return { start: offsetToPosition(source, at), end: offsetToPosition(source, at + length) };
}
