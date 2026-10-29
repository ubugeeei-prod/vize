const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createHash } = require("node:crypto");
const vscode = require("vscode");
const { assertPackagedExtension } = require("./real-server-support.cjs");
const { waitFor } = require("./wait-for.cjs");
const {
  assertCapture,
  capture,
  diagnostics,
  frames,
  publication,
  records,
  retain,
} = require("./router-host-packets.cjs");

const { prepareLocalResolution } = require("./router-local-resolve.cjs");

const caseName = (document) =>
  document.fileName.includes(`${path.sep}art-variants${path.sep}`)
    ? "art-variants"
    : document.languageId;

exports.run = async function run() {
  const manifest = JSON.parse(fs.readFileSync(process.env.VIZE_TEST_ROUTER_CAMPAIGN, "utf8"));
  const extension = vscode.extensions.getExtension("ubugeeei.vize");
  assert.ok(extension);
  assertPackagedExtension(extension);
  retain("installed-extension", {
    path: extension.extensionPath,
    packageJSON: extension.packageJSON,
    bundleSha256: createHash("sha256")
      .update(fs.readFileSync(path.join(extension.extensionPath, extension.packageJSON.main)))
      .digest("hex"),
    host: { version: vscode.version, appRoot: vscode.env.appRoot, versions: process.versions },
  });
  assert.equal(vscode.version, "1.107.1");
  await extension.activate();
  const configuration = vscode.workspace.getConfiguration("vize");
  const settings = Object.fromEntries(
    ["serverPath", "typecheck.enable", "editor.enable", "ecosystem.enable"].map((key) => [
      key,
      configuration.inspect(key),
    ]),
  );
  retain("client-settings", settings);
  assert.equal(configuration.get("serverPath"), manifest.launcher);
  for (const key of ["typecheck.enable", "editor.enable", "ecosystem.enable"]) {
    assert.equal(settings[key].workspaceValue, undefined);
    assert.equal(configuration.get(key), true);
  }
  const failures = [];
  const documents = new Map();
  const baseline = new Map();
  const authored = new Map();
  const immutable = new Map(
    manifest.projects
      .flatMap((project) => [
        path.join(project.root, "typed-router.d.ts"),
        path.join(project.root, "vite.config.mjs"),
        ...(project.enabled ? [] : [project.config]),
      ])
      .map((file) => [file, fs.readFileSync(file)]),
  );
  const test = async (label, operation) => {
    try {
      await operation();
    } catch (error) {
      failures.push({ label, message: error.message, stack: error.stack });
      retain("failures", failures);
    }
  };
  for (const project of manifest.projects) {
    for (const file of project.pages) {
      const document = await vscode.workspace.openTextDocument(file);
      // Pin the buffers: replacing a preview tab may close its document and
      // would remove that project's real watched-publication control.
      await vscode.window.showTextDocument(document, { preview: false });
      assert.equal(document.languageId, file.endsWith(".art.vue") ? "art-vue" : "vue");
      documents.set(file, document);
      authored.set(file, document.getText());
      const label = `${project.name}-${caseName(document)}`;
      await test(label, async () => {
        const event = await publication(document);
        retain(`${label}-publication`, event);
        const value = await capture(document, label);
        baseline.set(file, value);
        assert.deepEqual(event.packet.params, {
          diagnostics: diagnostics(document, project.enabled, false),
          uri: document.uri.toString(),
          version: document.version,
        });
        assertCapture(value, project, document, project.enabled);
      });
    }
  }
  const initial = frames().find(
    ({ direction, packet }) => direction === "client" && packet.method === "initialize",
  );
  retain("initialize", initial);
  assert.deepEqual(
    initial.packet.params.workspaceFolders.map(({ uri }) => uri),
    manifest.projects.map(({ root }) => vscode.Uri.file(root).toString()),
  );
  assert.equal(
    initial.packet.params.capabilities.workspace.didChangeWatchedFiles.dynamicRegistration,
    true,
  );
  assert.equal(
    initial.packet.params.capabilities.workspace.didChangeWatchedFiles.relativePatternSupport,
    true,
  );
  const selected = await vscode.commands.executeCommand("vize.test.getServerInfo");
  retain("selected-server", selected);
  assert.equal(selected.path, manifest.launcher);
  assert.equal(selected.extensionVersion, extension.packageJSON.version);
  const [enabled, disabled] = manifest.projects;
  for (const document of documents.values()) assert.equal(document.isClosed, false);

  let localResolution;
  await test("local-resolution-initial", async () => {
    localResolution = await prepareLocalResolution(manifest);
  });

  for (const file of enabled.pages) {
    const document = documents.get(file);
    const valid = authored.get(file);
    for (const invalid of [true, false])
      await test(`${caseName(document)}-${invalid ? "invalid" : "repair"}`, async () => {
        const start = frames().length;
        const text = invalid
          ? valid.replace("route.params.page.toFixed()", "route.params.page.toUpperCase()")
          : valid;
        const edit = new vscode.WorkspaceEdit();
        edit.replace(
          document.uri,
          new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)),
          text,
        );
        assert.equal(await vscode.workspace.applyEdit(edit), true);
        const event = await publication(document, start);
        retain(`${caseName(document)}-${invalid ? "invalid" : "repair"}-publication`, event);
        const value = await capture(
          document,
          `${caseName(document)}-${invalid ? "invalid" : "repair"}`,
        );
        assert.deepEqual(event.packet.params, {
          diagnostics: diagnostics(document, true, invalid),
          uri: document.uri.toString(),
          version: document.version,
        });
        assertCapture(value, enabled, document, true, invalid);
      });
  }

  for (const on of [false, true])
    await test(`config-${on ? "on" : "off"}`, async () => {
      const start = frames().length;
      const config = JSON.parse(fs.readFileSync(enabled.config, "utf8"));
      config.vueCompilerOptions.plugins = on ? ["vue-router/volar/sfc-typed-router"] : [];
      fs.writeFileSync(enabled.config, `${JSON.stringify(config, null, 2)}\n`);
      retain(`config-${on ? "on" : "off"}`, {
        path: enabled.config,
        source: fs.readFileSync(enabled.config, "utf8"),
      });
      const notification = await waitFor(
        () =>
          frames()
            .slice(start)
            .find(
              ({ direction, packet }) =>
                direction === "client" &&
                packet.method === "workspace/didChangeWatchedFiles" &&
                packet.params.changes.some(
                  ({ uri }) => uri === vscode.Uri.file(enabled.config).toString(),
                ),
            ),
        Boolean,
        "filesystem-only tsconfig watcher delivery",
        30_000,
      );
      retain(`config-${on ? "on" : "off"}-watch`, notification);
      const after =
        frames().findIndex(
          (record, index) => index >= start && record.frameBase64 === notification.frameBase64,
        ) + 1;
      for (const file of enabled.pages) {
        const document = documents.get(file);
        const event = await publication(document, after);
        retain(`config-${on ? "on" : "off"}-${caseName(document)}-publication`, event);
        await test(`config-${on ? "on" : "off"}-${caseName(document)}-publication`, async () =>
          assert.deepEqual(event.packet.params, {
            diagnostics: diagnostics(document, on, false),
            uri: document.uri.toString(),
            version: document.version,
          }));
        const terminal = frames().findIndex(
          (record, index) => index >= after && record.frameBase64 === event.frameBase64,
        );
        assert.deepEqual(
          frames()
            .slice(start, terminal + 1)
            .filter(
              ({ direction, packet }) =>
                direction === "client" &&
                packet.id !== undefined &&
                packet.params?.textDocument?.uri?.startsWith(
                  `${vscode.Uri.file(enabled.root).toString()}/`,
                ),
            ),
          [],
          "native config watch publishes before any new A-owner typed query",
        );
      }
      for (const file of enabled.pages)
        await test(`config-${on ? "on" : "off"}-${caseName(documents.get(file))}`, async () =>
          assertCapture(
            await capture(
              documents.get(file),
              `config-${on ? "on" : "off"}-${caseName(documents.get(file))}`,
            ),
            enabled,
            documents.get(file),
            on,
          ));
      for (const file of disabled.pages) {
        const document = documents.get(file);
        await test(`sibling-${on ? "on" : "off"}-${caseName(document)}`, async () => {
          const value = await capture(
            document,
            `sibling-${on ? "on" : "off"}-${caseName(document)}`,
          );
          assertCapture(value, disabled, document, false);
          assert.deepEqual(
            value.native,
            baseline.get(file).native,
            "sibling native response vector remains byte-for-byte equivalent",
          );
          assert.deepEqual(
            value.public,
            baseline.get(file).public,
            "sibling whole public editor result remains equivalent",
          );
        });
      }
      await test(`local-resolution-${on ? "on" : "off"}`, async () => {
        assert.ok(localResolution);
        await localResolution.afterWatch(on, after);
      });
    });
  const allSpawns = records().filter(({ kind }) => kind === "spawn");
  const spawns = allSpawns.filter(({ argv }) => argv[0] === "lsp");
  retain("process-isolation", { spawns, allSpawns, failures });
  assert.equal(spawns.length, 1, "config edits retain the same outer native server process");
  assert.ok(frames().every(({ pid }) => pid === spawns[0].pid));
  for (const [file, bytes] of immutable) assert.deepEqual(fs.readFileSync(file), bytes);
  for (const [file, source] of authored) {
    assert.equal(
      fs.readFileSync(file, "utf8"),
      source,
      "buffer-only diagnostic edits preserve every authored source byte",
    );
    assert.equal(documents.get(file).getText(), source);
  }
  assert.deepEqual(failures, []);
  await vscode.commands.executeCommand("vize.disable");
};
