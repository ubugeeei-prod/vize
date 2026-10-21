const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { createHash } = require("node:crypto");
const vscode = require("vscode");
const {
  describeDiagnostic,
  positionAfter,
  waitForDiagnostics,
} = require("./real-server-support.cjs");
const { waitFor } = require("./wait-for.cjs");

exports.run = async function run() {
  const manifest = JSON.parse(
    fs.readFileSync(process.env.VIZE_TEST_EXTERNAL_CONFIG_MANIFEST, "utf8"),
  );
  const { policy, pending, base, inherited, a, b, vite } = manifest;
  const workspace = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const uri = (file) => vscode.Uri.file(file).toString();
  const records = () => {
    const bytes = fs.readFileSync(process.env.VIZE_TEST_CONFIG_PROTOCOL_PATH, "utf8");
    return bytes
      .slice(0, bytes.lastIndexOf("\n") + 1)
      .split("\n")
      .filter(Boolean)
      .map(JSON.parse);
  };
  const frames = () => records().filter((record) => record.kind === "frame");
  const probe = await vscode.workspace.openTextDocument(path.join(a, "src/Probe.vue"));
  const control = await vscode.workspace.openTextDocument(path.join(b, "src/Control.vue"));
  const authored = new Map(
    [
      probe.fileName,
      control.fileName,
      path.join(b, "vite.config.mjs"),
      path.join(b, "tsconfig.json"),
    ].map((file) => [file, fs.readFileSync(file)]),
  );
  await vscode.window.showTextDocument(probe);
  const expected = (textPolicy, nullable) => [
    ...(textPolicy
      ? [diagnostic(probe, "typed", "Type 'number' is not assignable to type 'string'.")]
      : []),
    ...(nullable
      ? [diagnostic(probe, "nullable", "Type 'null' is not assignable to type 'string'.")]
      : []),
  ];
  const assertDiagnostics = async (values) => {
    const actual = await waitForDiagnostics(
      probe.uri,
      (items) => JSON.stringify(items.map(describeDiagnostic)) === JSON.stringify(values),
      "the complete native outside-config diagnostic vector",
      30_000,
    );
    assert.deepEqual(actual.map(describeDiagnostic), values);
  };
  await assertDiagnostics(expected(false, true));
  await waitForDiagnostics(
    control.uri,
    (values) => values.length === 0,
    "independent sibling owner",
    30_000,
  );
  const hover = (document, needle, prefix) =>
    vscode.commands.executeCommand("vize.test.executeLspRequest", {
      method: "textDocument/hover",
      params: {
        textDocument: { uri: document.uri.toString() },
        position: positionAfter(document, needle, prefix),
      },
    });
  const controlHover = () => hover(control, "const stable", "const sta");
  const baseline = await waitFor(
    controlHover,
    (value) => value != null,
    "real sibling typed hover",
    30_000,
  );
  const policyHover = () => hover(probe, "const typed", "const ty");
  assert.equal(await policyHover(), null, "the outside helper initially disables hover");
  await vscode.window.showTextDocument(control);
  const initial = frames().find(
    (record) => record.direction === "client" && record.packet.method === "initialize",
  );
  assert.deepEqual(
    initial.packet.params.workspaceFolders.map((folder) => folder.uri),
    [uri(workspace)],
  );
  const outside = [policy, base, inherited];
  for (const file of outside) {
    assert.equal(path.relative(workspace, file).startsWith(".."), true);
    await registered(file);
  }

  async function registered(file, start = 0) {
    const parent = vscode.Uri.file(path.dirname(file)).toString();
    const escaped = Array.from(path.basename(file))
      .map(
        (character) =>
          ({
            "[": "[[]",
            "]": "[]]",
            "{": "[{]",
            "}": "[}]",
            "?": "[?]",
            "*": "[*]",
            " ": "[ ]",
          })[character] ?? character,
      )
      .join("");
    await waitFor(
      () =>
        frames()
          .slice(start)
          .find(
            (record) =>
              record.direction === "server" &&
              record.packet.method === "client/registerCapability" &&
              record.packet.params.registrations.some((registration) =>
                registration.registerOptions.watchers.some(
                  (watch) =>
                    typeof watch.globPattern === "object" &&
                    watch.globPattern.baseUri.replace(/\/$/, "") === parent.replace(/\/$/, "") &&
                    watch.globPattern.pattern === escaped &&
                    watch.kind === 7,
                ),
              ),
          ),
      Boolean,
      `native exact RelativePattern registration for ${file}`,
      30_000,
    );
  }

  async function change(file, write, values, rejected = false) {
    const start = frames().length;
    write();
    const notification = await waitFor(
      () =>
        frames()
          .slice(start)
          .find(
            (record) =>
              record.direction === "client" &&
              record.packet.method === "workspace/didChangeWatchedFiles" &&
              record.packet.params.changes.some((change) => change.uri === uri(file)),
          ),
      Boolean,
      `real VS Code OS event for ${file}`,
      30_000,
    );
    const observed = frames();
    const after =
      observed.findIndex(
        (record, index) => index >= start && record.frameBase64 === notification.frameBase64,
      ) + 1;
    if (rejected) {
      await waitFor(
        () =>
          frames()
            .slice(after)
            .find(
              (record) =>
                record.direction === "server" &&
                record.packet.method === "window/showMessage" &&
                record.packet.params.message.includes("Keeping the last good configuration"),
            ),
        Boolean,
        "native failed reload retains its admitted owner",
        30_000,
      );
    }
    const publication = await waitFor(
      () =>
        frames()
          .slice(after)
          .find(
            (record) =>
              record.direction === "server" &&
              record.packet.method === "textDocument/publishDiagnostics" &&
              record.packet.params.uri === probe.uri.toString() &&
              record.packet.params.diagnostics.length === values.length,
          ),
      Boolean,
      "OS notification triggers native publication before any hover request",
      30_000,
    );
    const terminal = frames().findIndex(
      (record, index) => index >= after && record.frameBase64 === publication.frameBase64,
    );
    assert.deepEqual(
      frames()
        .slice(start, terminal + 1)
        .filter(
          (record) =>
            record.direction === "client" &&
            record.packet.id !== undefined &&
            record.packet.params?.textDocument?.uri?.startsWith(`${uri(a)}/`),
        ),
      [],
      "the outside watch must publish before any A-owner request can revalidate configuration",
    );
    await assertDiagnostics(values);
    assert.deepEqual(
      await controlHover(),
      baseline,
      "the entire independent owner's native hover is unchanged",
    );
    assert.deepEqual(vscode.languages.getDiagnostics(control.uri).map(describeDiagnostic), []);
    for (const [file, bytes] of authored) assert.deepEqual(fs.readFileSync(file), bytes);
  }

  const policySource = (enabled, textPolicy = false) =>
    `export default ${JSON.stringify({ languageServer: { hover: enabled }, typeChecker: textPolicy ? { tsconfig: "./tsconfig.text.json" } : {}, linter: { enabled: false } })};\n`;
  await change(
    policy,
    () => fs.writeFileSync(policy, policySource(true, true)),
    expected(true, true),
  );
  const enabled = await policyHover();
  assert.notEqual(enabled, null, "the imported outside helper changes actual native hover policy");
  await change(
    policy,
    () => fs.writeFileSync(policy, "export default {"),
    expected(true, true),
    true,
  );
  assert.deepEqual(
    await policyHover(),
    enabled,
    "invalid helper bytes keep the whole last good native hover",
  );
  await change(policy, () => fs.writeFileSync(policy, policySource(false)), expected(false, true));
  assert.equal(await policyHover(), null);
  const numericBase = fs.readFileSync(base, "utf8");
  const textBase = JSON.parse(numericBase);
  textBase.compilerOptions.paths["@policy"] = ["./types/string.ts"];
  await change(base, () => fs.writeFileSync(base, JSON.stringify(textBase)), expected(true, true));
  const config = path.join(a, "tsconfig.json");
  const missingExtends = JSON.parse(fs.readFileSync(config, "utf8")).extends[0];
  await change(base, () => fs.rmSync(base), [
    diagnosticAt(
      probe,
      '"@policy"',
      2307,
      "Cannot find module '@policy' or its corresponding type declarations.",
    ),
    ...expected(false, true),
    diagnosticAt(
      probe,
      "",
      "configuration-error",
      `Configuration error: cannot resolve tsconfig extends ${JSON.stringify(missingExtends)} from ${config}`,
    ),
  ]);
  await change(base, () => fs.writeFileSync(base, numericBase), expected(false, true));
  await change(
    inherited,
    () => fs.writeFileSync(inherited, '{"compilerOptions":{"strictNullChecks":false}}'),
    [],
  );
  const missingStart = frames().length;
  await change(
    vite,
    () =>
      fs.writeFileSync(
        vite,
        "import vue from '../../../new policy.mjs'; export default { vue };\n",
      ),
    [],
    true,
  );
  await registered(pending, missingStart);
  await change(
    pending,
    () => fs.writeFileSync(pending, policySource(true, true)),
    expected(true, false),
  );
  assert.notEqual(
    await policyHover(),
    null,
    "creating the new outside import repairs the failed reload automatically",
  );

  const spawns = records().filter((record) => record.kind === "spawn" && record.argv[0] === "lsp");
  assert.equal(spawns.length, 1, "external native config changes never restart the LSP process");
  assert.equal(spawns[0].binary, process.env.VIZE_TEST_OBSERVED_SERVER_PATH);
  assert.equal(
    spawns[0].binarySha256,
    createHash("sha256").update(fs.readFileSync(spawns[0].binary)).digest("hex"),
  );
  assert.ok(frames().every((record) => record.pid === spawns[0].pid));
  console.log(
    `[vize-host-real] external config OS watches, full native A/B vectors and failed-target repair passed pid=${spawns[0].pid} sha256=${spawns[0].binarySha256}`,
  );
};

function diagnostic(document, binding, message) {
  const offset = document.getText().indexOf(`const ${binding}`) + "const ".length;
  const start = document.positionAt(offset);
  const end = document.positionAt(offset + binding.length);
  return {
    code: 2322,
    message,
    range: [start.line, start.character, end.line, end.character],
    relatedInformation: undefined,
    severity: vscode.DiagnosticSeverity.Error,
    source: "vize/types",
    tags: undefined,
  };
}

function diagnosticAt(document, needle, code, message) {
  const offset = needle === "" ? 0 : document.getText().indexOf(needle);
  assert.notEqual(offset, -1, `fixture must contain ${JSON.stringify(needle)}`);
  const start = document.positionAt(offset);
  const end = document.positionAt(offset + needle.length);
  return {
    code,
    message,
    range: [start.line, start.character, end.line, end.character],
    relatedInformation: undefined,
    severity: vscode.DiagnosticSeverity.Error,
    source: "vize/types",
    tags: undefined,
  };
}
