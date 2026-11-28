import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

/** Each component traverses a distinct barrel, exercising dependency-string misses. */
export function separateDependencies(root, modules) {
  const components = path.join(root, "src/components");
  const expected = {};
  for (const name of fs.readdirSync(components).sort()) {
    assert(name.endsWith(".vue"));
    const old = path.join(components, name);
    const directory = path.join(components, path.basename(name, ".vue"));
    fs.mkdirSync(path.join(directory, "models"), { recursive: true });
    const file = path.join(directory, name);
    const source = fs.readFileSync(old, "utf8");
    assert(source.includes('"../types"'));
    fs.writeFileSync(file, source.replace('"../types"', '"./types"'));
    fs.rmSync(old);
    fs.writeFileSync(
      path.join(directory, "types.ts"),
      'import type { Model0 } from "./models";\nexport interface ItemProps extends Model0 { id: string; label: string }\n',
    );
    const modelPaths = Array.from({ length: modules }, (_, index) => {
      const model = path.join(directory, `models/Model${index}.ts`);
      fs.writeFileSync(model, `export interface Model${index} { field${index}: string }\n`);
      return model;
    });
    fs.writeFileSync(
      path.join(directory, "models/index.ts"),
      modelPaths.map((_, index) => `export * from "./Model${index}";\n`).join(""),
    );
    expected[file] = [
      path.join(root, "tsconfig.json"),
      path.join(root, "src/tsconfig.json"),
      path.join(components, "tsconfig.json"),
      path.join(directory, "tsconfig.json"),
      path.join(directory, "types.ts"),
      path.join(directory, "models/index.ts"),
      ...modelPaths,
    ].sort();
  }
  return expected;
}
