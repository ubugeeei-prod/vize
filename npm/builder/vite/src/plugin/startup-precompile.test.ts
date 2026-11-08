import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { build, type Plugin } from "vite";
import { vize } from "./index.ts";
import { compileAll, PrecompileDiagnosticsError } from "./precompile-run.ts";
import { compileFile, formatCompileErrorMessage, VizeSfcCompileError } from "../compiler.ts";
import { loadSfcModule } from "../test/sfc-module.ts";
import { toVirtualId } from "../virtual.ts";
import { makeProject, makeState, persistedFiles } from "../test/precompile-cache-project.ts";
import {
  linkWorkspaceVue,
  observeConsole,
  reportedProject,
  retainObservation,
  startupHost,
} from "../test/startup-scan-project.ts";

const missingBlock = [
  "At least one <template> or <script> is required in a single file component.",
];
const repaired = "<template><p>implemented</p></template>\n";

void test("reported unimported SFC diagnostics defer in build and serve, imports remain strict", async () => {
  for (const command of ["build", "serve"] as const) {
    const project = reportedProject(11375);
    const host = await startupHost(project.root, { plugins: vize({ configMode: false }) }, command);
    try {
      const start = await observeConsole(() => host.start());
      const imports = [
        await observeConsole(() => host.load(project.file("src/Placeholder.vue"), false)),
        await observeConsole(() => host.load(project.file("src/Placeholder.vue"), true)),
      ];
      const app = await observeConsole(() => host.load(project.file("src/App.vue")));
      retainObservation(command, { fixture: project.fixture, start, imports, app });
      assert.equal(start.error, null);
      const message = formatCompileErrorMessage(project.file("src/Placeholder.vue"), missingBlock);
      assert.deepEqual(start.calls.warn, [
        [
          "[vize]",
          `[vize] Pre-compilation failed for 1 file(s).\n\n${message}\nStartup scan diagnostics are deferred until import.`,
        ],
      ]);
      for (const imported of imports) {
        assert.ok(imported.error instanceof VizeSfcCompileError);
        assert.equal(imported.error.message, message);
        assert.deepEqual(imported.error.diagnostics, missingBlock);
        assert.equal(imported.error.filePath, project.file("src/Placeholder.vue"));
      }
      assert.equal(app.error, null);
      assert.ok(app.value);
      fs.writeFileSync(project.file("src/Placeholder.vue"), repaired);
      const repairedImports = [
        await observeConsole(() => host.load(project.file("src/Placeholder.vue"), false)),
        await observeConsole(() => host.load(project.file("src/Placeholder.vue"), true)),
      ];
      retainObservation(`${command}-repair`, repairedImports);
      for (const imported of repairedImports) {
        assert.equal(imported.error, null);
        assert.ok(imported.value);
      }
    } finally {
      await host.close();
      project.cleanup();
    }
  }
});

void test("failed cold and warm scans evict actual client, SSR, CSS and disk entries", async () => {
  const project = reportedProject(11375);
  const file = project.file("src/Placeholder.vue");
  const original = fs.readFileSync(file, "utf8");
  try {
    fs.writeFileSync(file, repaired);
    const state = makeState(project.root);
    await compileAll(state);
    compileFile(file, state.ssrCache, { ssr: true });
    state.collectedCss.set(file, "previous style");
    assert.equal(state.cache.has(file), true);
    assert.equal(state.ssrCache.has(file), true);
    assert.equal(persistedFiles(project.root).includes(file), true);
    fs.writeFileSync(file, original);
    for (const current of [state, makeState(project.root), makeState(project.root)]) {
      const scan = await observeConsole(() => compileAll(current));
      const cached = {
        client: [...current.cache],
        ssr: [...current.ssrCache],
        css: [...current.collectedCss],
        metadata: [...current.precompileMetadata],
        disk: persistedFiles(project.root),
      };
      const imports = await Promise.all(
        [false, true].map(async (ssr) => {
          try {
            const result = await loadSfcModule(current, toVirtualId(file, ssr), { ssr });
            return { result, error: null };
          } catch (error) {
            return { result: null, error };
          }
        }),
      );
      retainObservation("cache-refusal", { original, scan, cached, imports });
      assert.ok(scan.error instanceof PrecompileDiagnosticsError);
      assert.deepEqual(
        [
          current.cache.has(file),
          current.ssrCache.has(file),
          current.collectedCss.has(file),
          current.precompileMetadata.has(file),
          cached.disk.includes(file),
        ],
        [false, false, false, false, false],
      );
      assert.deepEqual(cached.disk, [project.file("src/App.vue")]);
      for (const imported of imports) assert.ok(imported.error instanceof VizeSfcCompileError);
    }
    fs.writeFileSync(file, repaired);
    const restored = makeState(project.root);
    const scan = await observeConsole(() => compileAll(restored));
    retainObservation("cache-repair", {
      scan,
      entries: [...restored.cache],
      disk: persistedFiles(project.root),
    });
    assert.equal(scan.error, null);
    assert.deepEqual(persistedFiles(project.root), [project.file("src/App.vue"), file]);
  } finally {
    project.cleanup();
  }
});

void test("SSR buildStart precompiles into the SSR cache and transform reuses it", async () => {
  const project = makeProject(
    `<script setup>const message = "server"</script><template><p>{{ message }}</p></template>`,
  );
  const host = await startupHost(project.root, {
    plugins: vize({ configMode: false, debug: true }),
    build: { ssr: true },
  });
  try {
    const observed = await observeConsole(async () => {
      await host.start();
      return host.load(project.file, true);
    });
    assert.equal(observed.error, null);
    assert.ok(observed.value);
    assert.equal(
      observed.calls.log.some((args) => args.join(" ").includes("transform: compiling")),
      false,
      "the SSR transform must not compile a second time after startup",
    );
  } finally {
    await host.close();
  }
});

void test("real Vite graph ignores an unused placeholder but rejects its actual import until repaired", async () => {
  const project = reportedProject(11375);
  try {
    const provider = linkWorkspaceVue(project.root);
    const bundle = () =>
      build({
        root: project.root,
        configFile: false,
        plugins: vize({ configMode: false }),
        build: { write: false, minify: false },
      });
    const unused = await observeConsole(bundle);
    fs.writeFileSync(
      project.file("src/main.ts"),
      'import "./Placeholder.vue";\n' + fs.readFileSync(project.file("src/main.ts"), "utf8"),
    );
    const imported = await observeConsole(bundle);
    fs.writeFileSync(project.file("src/Placeholder.vue"), repaired);
    const repair = await observeConsole(bundle);
    retainObservation("actual-vite-graph", {
      fixture: project.fixture,
      provider,
      unused,
      imported,
      repair,
    });
    assert.equal(unused.error, null);
    assert.ok(unused.value);
    assert.ok(imported.error instanceof Error);
    assert.match(
      imported.error.message,
      /At least one <template> or <script> is required in a single file component\./,
    );
    assert.equal(repair.error, null);
    assert.ok(repair.value);
  } finally {
    project.cleanup();
  }
});

void test("startup preserves owner exceptions and missing external source failures", async () => {
  const project = reportedProject(11375);
  const sentinel = new Error("actual companion failure");
  const companion: Plugin = {
    name: "test:throwing-owner",
    api: {
      vizePrecompileInputOwner: {
        version: 1,
        forConfig: () => ({
          ownsInput() {
            throw sentinel;
          },
        }),
      },
    },
  };
  try {
    const host = await startupHost(project.root, {
      plugins: [...vize({ configMode: false }), companion],
    });
    const owner = await observeConsole(() => host.start());
    await host.close();
    fs.writeFileSync(
      project.file("src/Placeholder.vue"),
      '<template src="./missing.html"></template>\n',
    );
    const missing = await startupHost(project.root, { plugins: vize({ configMode: false }) });
    const source = await observeConsole(() => missing.start());
    await missing.close();
    retainObservation("fatal-nondiagnostics", { owner, source });
    assert.equal(owner.error, sentinel);
    assert.ok(source.error instanceof Error);
    assert.equal(source.error instanceof PrecompileDiagnosticsError, false);
    assert.equal(
      source.error.message,
      `[vize] <template src="./missing.html"> not found (resolved: ${project.file("src/missing.html")}) in ${project.file("src/Placeholder.vue")}`,
    );
    assert.deepEqual(source.calls.warn, []);
  } finally {
    project.cleanup();
  }
});

void test("a read race mixed with known compiler diagnostics remains a fatal whole aggregate", async () => {
  const project = reportedProject(11375);
  // The real owner callback runs after glob/stat and before the next actual read.
  // Three candidates ensure a diagnostic survives regardless of glob ordering.
  fs.writeFileSync(project.file("src/App.vue"), "<!-- invalid App -->\n");
  fs.writeFileSync(project.file("src/Third.vue"), "<!-- invalid Third -->\n");
  const files = ["src/App.vue", "src/Placeholder.vue", "src/Third.vue"].map(project.file);
  let removed: string | undefined;
  const race: Plugin = {
    name: "test:read-race",
    api: {
      vizePrecompileInputOwner: {
        version: 1,
        forConfig: () => ({
          ownsInput(file: string) {
            if (!removed) {
              removed = files.find(
                (candidate) => path.normalize(candidate) !== path.normalize(file),
              )!;
              fs.unlinkSync(removed);
            }
            return false;
          },
        }),
      },
    },
  };
  try {
    const host = await startupHost(project.root, {
      plugins: [...vize({ configMode: false }), race],
    });
    const observed = await observeConsole(() => host.start());
    await host.close();
    retainObservation("mixed-read-failure", { removed, observed });
    assert.ok(removed);
    assert.ok(observed.error instanceof Error);
    assert.equal(observed.error instanceof PrecompileDiagnosticsError, false);
    assert.match(observed.error.message, /^\[vize\] Pre-compilation failed for 3 file\(s\)\./);
    assert.match(observed.error.message, /Failed to read /);
    assert.match(observed.error.message, /At least one <template> or <script> is required/);
    assert.deepEqual(observed.calls.warn, []);
  } finally {
    project.cleanup();
  }
});
