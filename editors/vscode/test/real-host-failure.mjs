import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

// Only the controlled fixture files and its own server log survive cleanup.
// Never copy node_modules, the editor profile, or installed extensions.
const files = [
  "node_modules/.vize/lsp.log",
  "src/App.vue",
  "src/Clean.vue",
  "src/Child.vue",
  "src/Scenario.vue",
  "src/RefSurface.vue",
  "src/ContractChild.vue",
  "src/ContractHost.vue",
  "template/bare/typescript/src/App.vue",
  "vize.config.json",
  "tsconfig.json",
  ".vscode/settings.json",
];

export function retainRealHostFailure(workspacePath, destination, error) {
  const workspace = fs.realpathSync(workspacePath);
  fs.mkdirSync(destination, { recursive: true });
  const captured = [];
  const missing = [];
  for (const file of files) {
    const input = path.join(workspace, file);
    if (!fs.existsSync(input)) {
      missing.push(file);
      continue;
    }
    const relative = path.relative(workspace, fs.realpathSync(input));
    if (relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
      throw new Error(`fixture evidence escapes the workspace: ${file}`);
    }
    if (!fs.lstatSync(input).isFile()) {
      throw new Error(`fixture evidence is not a regular file: ${file}`);
    }
    const bytes = fs.readFileSync(input);
    const output = `${file}.txt`;
    fs.mkdirSync(path.dirname(path.join(destination, output)), {
      recursive: true,
    });
    fs.writeFileSync(path.join(destination, output), bytes);
    captured.push({
      file,
      output,
      bytes: bytes.length,
      sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
    });
  }
  const receipt = {
    schemaVersion: 1,
    error:
      error instanceof Error
        ? { name: error.name, message: error.message, stack: error.stack }
        : { message: String(error) },
    captured,
    missing,
    sourceCustody:
      "Full controlled files on disk at failure; unsaved editor buffers are not represented as disk bytes.",
  };
  fs.writeFileSync(
    path.join(destination, "manifest.json"),
    `${JSON.stringify(receipt, null, 2)}\n`,
  );
  return receipt;
}
