import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import type { CompiledModule } from "../types.ts";
import { makeState } from "../test/precompile-cache-project.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";
import { resolveIdHook } from "./resolve.ts";

function project(t: TestContext) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-deleted-sfc-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const file = path.join(root, "src/components/Item0.vue");
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, "<template />");
  return { root, components: [file] };
}

async function vueFileCalls(run: () => unknown) {
  const exists = new Map<string, number>();
  const original = fs.existsSync;
  fs.existsSync = (file) => {
    if (typeof file === "string" && file.endsWith(".vue"))
      exists.set(file, (exists.get(file) ?? 0) + 1);
    return original(file);
  };
  try {
    await run();
    return { exists };
  } finally {
    fs.existsSync = original;
  }
}

void test("a deleted cached SFC returns to the dev resolver before its watcher reports the deletion", async (t) => {
  const {
    root,
    components: [file],
  } = project(t);
  const compiled: CompiledModule = {
    code: "export default {}",
    scopeId: "deleted",
    hasScoped: false,
  };
  fs.unlinkSync(file);
  const upstream = Object.freeze({
    id: file,
    external: false,
    meta: { deletionProbe: true },
  });

  for (const kind of ["direct", "alias", "package"] as const) {
    const id =
      kind === "direct"
        ? "./components/Item0.vue"
        : kind === "alias"
          ? "@components/Item0.vue"
          : "#components/Item0.vue";
    const importer =
      kind === "package"
        ? toVirtualId(path.join(root, "src/Parent.vue"))
        : path.join(root, "src/main.ts");
    const context = { resolve: async () => (kind === "direct" ? null : upstream) };

    for (const cache of ["cache", "ssrCache"] as const) {
      for (const ssr of [false, true]) {
        for (const dev of [false, true]) {
          const state = makeState(root);
          state[cache].set(file, compiled);
          if (dev) state.server = {} as never;
          let resolved: unknown;
          const calls = await vueFileCalls(async () => {
            resolved = await resolveIdHook(context, state, id, importer, { ssr });
          });
          const label = `${kind}, ${cache}, ssr=${ssr}, dev=${dev}`;

          if (dev) {
            assert.equal(calls.exists.get(file), 1, label);
            // A package resolver's answer still belongs to the host, including
            // metadata the host needs; Vize must not replace it with an SFC id.
            assert.equal(resolved, kind === "package" ? upstream : null, label);
          } else {
            // A build deliberately keeps the snapshot it already compiled.
            assert.equal(calls.exists.get(file) ?? 0, 0, label);
            assert.equal(resolved, toPluginVisibleVirtualId(file, ssr), label);
          }
        }
      }
    }
  }
});
