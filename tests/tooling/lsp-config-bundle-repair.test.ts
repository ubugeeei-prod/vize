import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { LspWire } from "../differential/lsp-wire.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";

const require = createRequire(import.meta.url);

test(
  "an intermediate Vite helper repair applies on the next operation after its watched-file event",
  { timeout: 90_000 },
  async () => {
    const [binary] = resolveVizeLaunchCommand(undefined, path.join(root, "target/ci/vize"), {
      required: true,
      repoRoot: root,
    });
    const directory = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-bundle-boundary-")),
    );
    const app = path.join(directory, "app");
    const config = path.join(app, "vite.config.mjs");
    const bridge = path.join(directory, "bridge.mjs");
    const leaf = path.join(directory, "leaf.mjs");
    const missing = path.join(directory, "missing.mjs");
    const counter = path.join(directory, "evaluations.txt");
    const document = path.join(app, "App.vue");
    const source = "<template><div /></template>\n<style>.app { color:red }</style>";
    const factory = (policy: string, label: string) => `import { appendFileSync } from 'node:fs';
    export default () => { appendFileSync(${JSON.stringify(counter)}, ${JSON.stringify(label + "\n")});
      return { vue: ${policy} }; };`;
    const enabled = JSON.stringify({
      languageServer: { hover: true },
      linter: { enabled: false },
      typeChecker: { enabled: false },
    });
    const output = path.join(root, "target/differential/config-bundle-repair");
    let wire: LspWire | undefined;
    try {
      fs.mkdirSync(path.join(app, "node_modules"), { recursive: true });
      fs.symlinkSync(
        path.dirname(require.resolve("vite/package.json")),
        path.join(app, "node_modules/vite"),
        "junction",
      );
      fs.writeFileSync(path.join(app, "package.json"), '{"type":"module"}');
      fs.writeFileSync(document, source);
      fs.writeFileSync(config, factory(enabled, "initial"));
      wire = new LspWire(binary, ["lsp", "--stdio"], app);
      const initialized = await wire.request("initialize", {
        processId: null,
        rootUri: pathToFileURL(app).href,
        workspaceFolders: [{ uri: pathToFileURL(app).href, name: "app" }],
        capabilities: {},
        initializationOptions: { lint: false, typecheck: false, ecosystem: false },
      });
      assert.equal(initialized.error, undefined);
      wire.notify("initialized", {});
      const uri = pathToFileURL(document).href;
      wire.notify("textDocument/didOpen", {
        textDocument: { uri, languageId: "vue", version: 1, text: source },
      });
      await wire.waitFor(
        (packet) =>
          packet.method === "textDocument/publishDiagnostics" && packet.params?.uri === uri,
      );
      const hover = () =>
        wire!.request("textDocument/hover", {
          textDocument: { uri },
          position: { line: 1, character: 15 },
        });
      const first = await hover();
      assert.equal(first.error, undefined);
      assert.ok(first.result && typeof first.result === "object");
      const stable = await hover();
      assert.deepEqual(stable, { jsonrpc: "2.0", id: wire.nextId, result: first.result });
      const baseline = fs.readFileSync(counter, "utf8");
      fs.writeFileSync(bridge, "export { default } from './leaf.mjs';");
      fs.writeFileSync(leaf, "export { default } from './missing.mjs';");
      const broken = `import vue from '../bridge.mjs';\n${factory("vue", "repaired")}`;
      fs.writeFileSync(config, broken);
      const retained = await hover();
      assert.deepEqual(retained, { jsonrpc: "2.0", id: wire.nextId, result: first.result });
      assert.equal(
        fs.readFileSync(counter, "utf8"),
        baseline,
        "the failed bundle never executes the original factory",
      );
      const configBytes = fs.readFileSync(config);
      const leafBytes = fs.readFileSync(leaf);
      fs.writeFileSync(
        bridge,
        "export default { languageServer: { hover: false }, linter: { enabled: false }, typeChecker: { enabled: false } };",
      );
      // An unobservable failure is retried once per save or watched-file event (#9720).
      wire.notify("workspace/didChangeWatchedFiles", {
        changes: [{ uri: pathToFileURL(bridge).href, type: 2 }],
      });
      const repaired = await hover();
      assert.deepEqual(repaired, { jsonrpc: "2.0", id: wire.nextId, result: null });
      assert.equal(
        fs.readFileSync(counter, "utf8"),
        baseline + "repaired\n",
        "the next operation evaluates its factory exactly once",
      );
      const confirmed = await hover();
      assert.deepEqual(confirmed, { jsonrpc: "2.0", id: wire.nextId, result: null });
      const afterConfirmation = fs.readFileSync(counter, "utf8");
      assert.equal(
        afterConfirmation,
        baseline + "repaired\nrepaired\n",
        "one later operation confirms the newly discovered graph once",
      );
      const warm = await hover();
      assert.deepEqual(warm, { jsonrpc: "2.0", id: wire.nextId, result: null });
      assert.equal(
        fs.readFileSync(counter, "utf8"),
        afterConfirmation,
        "verified unchanged configuration performs no evaluation",
      );
      assert.deepEqual(fs.readFileSync(config), configBytes);
      assert.deepEqual(fs.readFileSync(leaf), leafBytes);
      assert.equal(fs.existsSync(missing), false);
      assert.equal(fs.readFileSync(document, "utf8"), source);
      await wire.finish();
    } finally {
      if (wire) {
        await wire.stop().catch(() => undefined);
        fs.mkdirSync(output, { recursive: true });
        const read = (file: string) => (fs.existsSync(file) ? fs.readFileSync(file, "utf8") : null);
        fs.writeFileSync(
          path.join(output, "observation.json"),
          JSON.stringify(
            {
              binary,
              binarySha256: createHash("sha256").update(fs.readFileSync(binary)).digest("hex"),
              source,
              config: read(config),
              bridge: read(bridge),
              leaf: read(leaf),
              evaluations: read(counter),
              ...wire.observation(),
            },
            null,
            2,
          ),
        );
      }
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
