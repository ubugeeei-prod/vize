const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vscode = require("vscode");
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
async function eventually(callback) {
  let last;
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    try {
      return await callback();
    } catch (error) {
      last = error;
      await pause(200);
    }
  }
  throw last;
}
async function run() {
  const workspace = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const uri = vscode.Uri.file(path.join(workspace, "src/main.ts"));
  const document = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(document);
  const extension = vscode.extensions.getExtension("ubugeeei.vize");
  assert(extension, "Installed VSIX must be present");
  const diagnostics = await eventually(() => {
    const values = vscode.languages.getDiagnostics(uri);
    assert(
      values.some((value) => [2322, 2741].includes(Number(value.code))),
      JSON.stringify(values),
    );
    assert(
      values.some((value) => Number(value.code) === 2307 && value.message.includes("Missing.vue")),
      JSON.stringify(values),
    );
    assert(
      !values.some((value) => Number(value.code) === 2307 && value.message.includes("App.vue")),
      JSON.stringify(values),
    );
    return values;
  });
  const text = document.getText();
  const offset = text.indexOf("props.title") + "props.".length + 1;
  const position = document.positionAt(offset);
  const hover = await eventually(async () => {
    const values = await vscode.commands.executeCommand(
      "vscode.executeHoverProvider",
      uri,
      position,
    );
    assert(values?.length > 0);
    assert(
      values.some((value) =>
        value.contents.some((content) =>
          /string/.test(typeof content === "string" ? content : content.value),
        ),
      ),
    );
    return values;
  });
  const completion = await eventually(async () => {
    const value = await vscode.commands.executeCommand(
      "vscode.executeCompletionItemProvider",
      uri,
      document.positionAt(text.indexOf("props.title") + "props.".length),
    );
    assert(
      value.items.some(
        (entry) => (typeof entry.label === "string" ? entry.label : entry.label.label) === "title",
      ),
    );
    return value;
  });
  const definition = await eventually(async () => {
    const values = await vscode.commands.executeCommand(
      "vscode.executeDefinitionProvider",
      uri,
      document.positionAt(text.indexOf("App from") + 1),
    );
    assert(
      values.some(
        (value) => (value.targetUri || value.uri).fsPath === path.join(workspace, "src/App.vue"),
      ),
    );
    return values;
  });
  const inheritedConfiguration = await checkInheritedConfiguration(workspace);
  fs.writeFileSync(
    process.env.VIZE_TYPESCRIPT_PROVIDER_RESULT,
    JSON.stringify(
      {
        extensionPath: extension.extensionPath,
        diagnostics,
        hover,
        completion,
        definition,
        inheritedConfiguration,
      },
      null,
      2,
    ),
  );
}

async function checkInheritedConfiguration(workspace) {
  const crypto = require("node:crypto");
  const { files, expected, pack } =
    require("../../../../tests/typescript-plugin/project-fixture.cjs").readProjectFixture();
  const root = path.join(workspace, "継承設定");
  const src = path.join(root, "src");
  fs.mkdirSync(src, { recursive: true });
  const config = path.join(root, "tsconfig.json");
  const base = path.join(root, "inherited.json");
  const vue = path.join(src, "Inherited.vue");
  const control = path.join(src, "control.ts");
  const entry = path.join(src, "entry.ts");
  const inputs = {
    config: files.get("tsconfig-5.7.json.txt"),
    vue: files.get("src/Inherited.vue.txt"),
    control: files.get("src/control.ts.txt"),
    entry: 'import "./Inherited.vue";\nexport const marker = 1;\n',
    before: JSON.parse(files.get("inherited-before.json.txt")),
    after: JSON.parse(files.get("inherited-after.json.txt")),
  };
  for (const [file, text] of [
    [config, inputs.config],
    [vue, inputs.vue],
    [control, inputs.control],
    [entry, inputs.entry],
  ])
    fs.writeFileSync(file, text);
  const fixed = [config, vue, control, entry].map((file) => ({
    file,
    sha256: crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"),
  }));
  const packet = { inputs, fixed, fixture: pack };
  const save = () =>
    fs.writeFileSync(
      `${process.env.VIZE_TYPESCRIPT_PROVIDER_RESULT}.configuration.json`,
      JSON.stringify(packet, null, 2),
    );
  const diagnostics = async (file) => {
    // The pinned built-in extension explicitly permits this read-only request.
    // Passing the actual path also includes closed mixed-content external files.
    const response = await vscode.commands.executeCommand(
      "typescript.tsserverRequest",
      "semanticDiagnosticsSync",
      { file },
    );
    assert.equal(response?.success, true, JSON.stringify(response));
    return response;
  };
  fs.writeFileSync(base, JSON.stringify(inputs.before));
  await vscode.window.showTextDocument(
    await vscode.workspace.openTextDocument(vscode.Uri.file(entry)),
  );
  try {
    packet.before = await eventually(async () => {
      const response = await diagnostics(vue);
      assert.deepEqual(response.body, []);
      return response;
    });
    packet.controlBefore = await diagnostics(control);
    assert.deepEqual(packet.controlBefore.body, []);
    save();
    fs.writeFileSync(base, JSON.stringify(inputs.after));
    packet.controlAfter = await eventually(async () => {
      const response = await diagnostics(control);
      assert(
        response.body.some((diagnostic) => diagnostic.code === 7029),
        JSON.stringify(response),
      );
      return response;
    });
    save();
    packet.warm = await eventually(async () => {
      const response = await diagnostics(vue);
      packet.latestWarm = response;
      save();
      assert(
        response.body.some(
          (diagnostic) =>
            diagnostic.code === 2339 && diagnostic.text.includes("UnknownClosureProbe"),
        ),
        JSON.stringify(response),
      );
      return response;
    });
    for (const input of fixed)
      assert.equal(
        crypto.createHash("sha256").update(fs.readFileSync(input.file)).digest("hex"),
        input.sha256,
        "only the inherited configuration changes",
      );
    await vscode.commands.executeCommand("typescript.restartTsServer");
    packet.fresh = await eventually(async () => {
      const response = await diagnostics(vue);
      packet.latestFresh = response;
      save();
      assert.deepEqual(response.body, packet.warm.body);
      return response;
    });
    assert.deepEqual(packet.fresh.body, expected.strict);
    assert.deepEqual(packet.controlAfter.body, expected.controlAfter);
    return packet;
  } finally {
    save();
  }
}
module.exports = { run };
