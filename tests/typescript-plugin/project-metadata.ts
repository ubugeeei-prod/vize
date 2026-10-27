import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

/** The installed native editor bindings these checks call. */
export interface NativeApi {
  getVueLanguageProjectMetadata(configFileName: string): {
    configIdentity: string;
    watchedFiles: string[];
  };
  generateVueLanguageCode(source: string, options: Record<string, unknown>): string;
}
/** The tsserver client from tsserver-client.mjs. */
export interface TsserverClient {
  send(command: string, args?: unknown, expectResponse?: boolean): Promise<any>;
  stop(): Promise<unknown>;
}

// This consumes the installed, exact-source native package. The inherited
// configuration fixture is independent of every existing protocol oracle.
const compareNames = (left: string, right: string) => (left < right ? -1 : left > right ? 1 : 0);

// Rust's path ordering compares components, so `preferred/tsconfig.json`
// precedes the sibling `preferred.json` even though whole-string order differs.
const comparePaths = (left: string, right: string) => {
  const a = left.split(path.sep),
    b = right.split(path.sep);
  for (let index = 0; index < Math.min(a.length, b.length); index++) {
    const compared = compareNames(a[index], b[index]);
    if (compared) return compared;
  }
  return a.length - b.length;
};

export function checkProjectMetadata(native: NativeApi, consumer: string, output: string) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(consumer, "metadata-")));
  const config = path.join(root, "tsconfig.json");
  const base = path.join(root, "base.json");
  const source =
    '<script setup lang="ts">const n = 1;</script><template><Unknown :n="n"/></template>';
  const packets: { name: string; metadata: unknown }[] = [];
  const inputs: { file: string; text: string }[] = [];
  const write = (file: string, text: string) => {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, text);
    inputs.push({ file, text });
  };
  const capture = (name: string, file = config) => {
    const metadata = native.getVueLanguageProjectMetadata(file);
    packets.push({ name, metadata });
    assert.deepEqual(Object.keys(metadata).sort(), ["configIdentity", "watchedFiles"]);
    assert.match(metadata.configIdentity, /^vize-content-mapper-v1:[a-f0-9]{64}$/);
    assert.deepEqual(metadata.watchedFiles, [...new Set(metadata.watchedFiles)].sort(comparePaths));
    assert(metadata.watchedFiles.every((file) => path.isAbsolute(file)));
    return metadata;
  };
  const generate = () =>
    native.generateVueLanguageCode(source, {
      filename: path.join(root, "Source.vue"),
      configFileName: config,
    });
  try {
    const inferred = capture("inferred", "");
    assert.deepEqual(inferred, {
      configIdentity:
        "vize-content-mapper-v1:7a38905296006c575d7d45d1d881690a3ee87361ed67ce64642e2504482da818",
      watchedFiles: [],
    });
    write(config, '{"extends":"./base.json"}');
    write(base, '{"vueCompilerOptions":{"strictTemplates":false}}');
    const before = capture("inherited-before");
    assert.deepEqual(before.watchedFiles, [base, config].sort(comparePaths));
    const beforeRaw = generate();
    assert.deepEqual(capture("unchanged"), before);
    assert.equal(generate(), beforeRaw, "Metadata reads leave the complete projection unchanged");
    write(base, '{"vueCompilerOptions":{"strictTemplates":true}}');
    const after = capture("inherited-after");
    assert.notEqual(after.configIdentity, before.configIdentity);
    assert.deepEqual(after.watchedFiles, before.watchedFiles);
    const afterRaw = generate();
    assert.notEqual(afterRaw, beforeRaw, "The same canonical options affect the real projection");
    write(
      base,
      '{"compilerOptions":{"strict":true},"vueCompilerOptions":{"strictTemplates":true}}',
    );
    assert.deepEqual(capture("ordinary-option-only"), after);
    assert.equal(generate(), afterRaw);
    const packageRoot = path.join(root, "node_modules/preset");
    const manifest = path.join(packageRoot, "package.json");
    const selected = path.join(packageRoot, "selected.json");
    write(manifest, '{"tsconfig":"selected.json"}');
    write(selected, '{"vueCompilerOptions":{"strictTemplates":false}}');
    write(config, '{"extends":"preset"}');
    const packaged = capture("package-extends");
    assert.deepEqual(packaged.watchedFiles, [config, manifest, selected].sort(comparePaths));
    const fallback = path.join(packageRoot, "tsconfig.json");
    write(manifest, '{"tsconfig":"preferred"}');
    write(fallback, '{"vueCompilerOptions":{"strictTemplates":false}}');
    const missing = capture("missing-preferred");
    const preferred = path.join(packageRoot, "preferred");
    assert(missing.watchedFiles.includes(preferred));
    assert(missing.watchedFiles.includes(`${preferred}.json`));
    assert(missing.watchedFiles.includes(fallback));
    write(`${preferred}.json`, '{"vueCompilerOptions":{"strictTemplates":true}}');
    const created = capture("preferred-created");
    assert.notEqual(created.configIdentity, missing.configIdentity);
    assert(created.watchedFiles.includes(`${preferred}.json`));
    assert(!created.watchedFiles.includes(fallback));
    write(config, '{"extends":"./base.json"}');
    write(base, "{ invalid JSON");
    assert.throws(() => native.getVueLanguageProjectMetadata(config), /Invalid tsconfig/);
    write(base, '{"vueCompilerOptions":{"strictTemplates":true}}');
    assert.deepEqual(capture("repaired"), after);
    fs.writeFileSync(
      path.join(output, "project-metadata-projections.json"),
      JSON.stringify({ source, beforeRaw, afterRaw }, null, 2),
    );
  } finally {
    fs.writeFileSync(
      path.join(output, "project-metadata-observations.json"),
      JSON.stringify({ inputs, packets }, null, 2),
    );
  }
}
