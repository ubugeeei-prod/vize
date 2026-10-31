/** Source execution requires the same source's complete physical declaration family. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sdk } from "./canon-slot-constructor-audit.ts";
import { isRecord } from "./canon-instance-directives-model.ts";
import { identify, save } from "./canon-instance-directives-io.ts";

export const sourceDeclarations = [
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
] as const;

export function sourceSdk(project: string, provider: string, binary: string): void {
  sdk(project, provider, binary);
  const installed = path.join(project, "node_modules/vize"),
    source = path.resolve("npm/cli");
  const manifest: unknown = JSON.parse(fs.readFileSync(path.join(source, "package.json"), "utf8"));
  assert.ok(
    isRecord(manifest) &&
      isRecord(manifest.typescript) &&
      isRecord(manifest.typescript.contentMapper),
  );
  manifest.typescript.contentMapper.exec = [binary, "content-mapper"];
  save(path.join(installed, "package.json"), manifest);
  const declarations = sourceDeclarations.map((file) => {
    const destination = path.join(installed, file);
    fs.rmSync(destination, { force: true });
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(source, file), destination);
    const original = identify(path.join(source, file)),
      copy = identify(destination);
    assert.equal(copy.bytes, original.bytes);
    assert.equal(copy.sha256, original.sha256);
    return { file, source: original, installed: copy };
  });
  save(path.join(project, "source-sdk-custody.json"), {
    package: identify(path.join(installed, "package.json")),
    declarations,
  });
}
