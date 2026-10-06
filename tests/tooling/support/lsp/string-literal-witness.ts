import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { offsetToPosition } from "./assertions.ts";

const markerName = "vize-string-literal-fixture";
const markerSource = (name: string): string =>
  `${JSON.stringify({ name, version: "0.0.0", private: true })}\n`;
export const hash = (bytes: string | Buffer): string =>
  createHash("sha256").update(bytes).digest("hex");

export function createMarker(workspace: string, name = markerName): void {
  const marker = path.join(workspace, "node_modules", name);
  fs.mkdirSync(marker);
  fs.writeFileSync(path.join(marker, "package.json"), markerSource(name));
}

function file(
  absolute: string,
  record: (row: unknown) => void,
): { path: string; source: string; sha256: string } {
  try {
    const source = fs.readFileSync(absolute, "utf8");
    const row = { path: absolute, source, sha256: hash(source) };
    record({ kind: "whole-witness-file", ...row });
    return row;
  } catch (error) {
    record({ kind: "witness-read-error", path: absolute, error: String(error) });
    throw error;
  }
}

// Establish physical ownership and the complete current generated source BEFORE
// reading a completion response. No observed opaque item supplies these values.
export function witness(
  workspace: string,
  source: string,
  messages: string,
  tsconfig: string,
  token: string,
  record: (row: unknown) => void,
  name = markerName,
): { requestUri: string; fileName: string; position: number; range: unknown } {
  const root = fs.realpathSync(workspace);
  const marker = path.join(root, "node_modules", name);
  assert.equal(file(path.join(marker, "package.json"), record).source, markerSource(name));
  const sessions = path.join(fs.realpathSync(os.tmpdir()), "vize-canon", "editor", "sessions");
  const matching: { mirror: string; link: string }[] = [];
  for (const session of fs.readdirSync(sessions, { withFileTypes: true })) {
    if (!session.isDirectory()) continue;
    const projects = path.join(sessions, session.name, "projects");
    if (!fs.existsSync(projects)) continue;
    for (const project of fs.readdirSync(projects, { withFileTypes: true })) {
      if (!project.isDirectory()) continue;
      const mirror = path.join(projects, project.name);
      const link = path.join(mirror, "node_modules", name);
      if (fs.existsSync(link) && fs.realpathSync(link) === marker) matching.push({ mirror, link });
    }
  }
  record({ kind: "mirror-candidates", root, marker, matching });
  assert.equal(matching.length, 1, "exactly one session namespace owns this physical marker");
  const { mirror, link } = matching[0];
  const generated = ["Page.vue.ts", "messages.ts", "tsconfig.json"].map((name) =>
    file(path.join(mirror, name), record),
  );
  const authored = ["Page.vue", "messages.ts", "tsconfig.json", "vize.config.json"].map((name) =>
    file(path.join(root, name), record),
  );
  record({
    kind: "whole-mirror",
    root,
    link,
    target: fs.readlinkSync(link),
    generated,
    authored,
    currentOverlay: source,
  });
  for (const entry of generated) {
    assert.ok(fs.lstatSync(entry.path).isFile());
    assert.equal(fs.realpathSync(entry.path), entry.path);
  }
  assert.equal(authored[1].source, messages);
  assert.equal(authored[2].source, tsconfig);
  assert.equal(generated[1].source, messages);
  assert.deepEqual(JSON.parse(generated[2].source), {
    compilerOptions: {
      module: "ESNext",
      moduleResolution: "Bundler",
      strict: true,
      noEmit: true,
      allowImportingTsExtensions: true,
      noUncheckedSideEffectImports: false,
    },
    exclude: [],
    include: ["Page.vue.ts", "__vize_vue_modules.d.ts", "messages.ts"],
  });
  const code = generated[0].source;
  assert.equal(source.split(token).length - 1, 1, "unique authored whole literal");
  assert.equal(code.split(token).length - 1, 1, "unique current generated whole literal");
  const start = code.indexOf(token) + 1;
  const end = start + token.length - 2;
  const range = { start: offsetToPosition(code, start), end: offsetToPosition(code, end) };
  const result = {
    requestUri: pathToFileURL(generated[0].path).href,
    fileName: generated[0].path,
    position: Buffer.byteLength(code.slice(0, start), "utf8"),
    range,
  };
  record({
    kind: "independent-query",
    token,
    utf16Offset: start,
    wholeGeneratedSha256: generated[0].sha256,
    ...result,
  });
  return result;
}
