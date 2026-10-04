import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import type { TestContext } from "node:test";

import {
  resolveVizeCommand,
  runVizeCheck,
  symlinkVueTypes,
} from "../../../_helpers/realworld-typecheck.ts";

import {
  type Topology,
  type Observation,
  ROOTS,
  SHARED,
  assertReport,
  assertOnlyInjectedCall,
} from "./partition-assertions.ts";

// Secondary synthetic visibility controls for #7817, never primary Router
// provider evidence. The parent oracle owns exact source CLI/LSP custody.
const HELPER = "node_modules/vue-router/auto-routes.d.mts";
const PLUGIN = "vue-router/volar/sfc-typed-router";
const GLOBALS = [
  "declare function useRoute<Name extends string = string>(name?: Name): { name: Name };",
  "interface UnrelatedPageGlobal { value: number; }",
  "declare const unrelatedPageValue: UnrelatedPageGlobal;",
  "",
].join("\n");
const ROUTES = [
  "export type _RouteNamesForFilePath<Path extends string> = 'current';",
  "declare global { interface RouteScopedGlobal { marker: 'route-global'; } }",
  "",
].join("\n");
const BARE_CALL = "const route = useRoute(); void route;";
const NEGATIVES = [
  ["no-call", "const route = 'no-call'; void route;"],
  ["explicit-argument", "const route = useRoute('other'); void route;"],
  ["explicit-type-argument", "const route = useRoute<'other'>(); void route;"],
  ["local-shadow", "const useRoute = () => 'local'; const route = useRoute(); void route;"],
] as const;

function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

function source(topology: Topology, statement: string, broken: boolean): string {
  const body =
    topology === "shared-leaf"
      ? [
          "import { snapshot } from './shared';",
          statement,
          "const scopedMismatch: string = snapshot;",
          "const unrelatedOk: number = unrelatedPageValue.value;",
          "const unrelatedMismatch: string = unrelatedPageValue.value;",
          "void scopedMismatch; void unrelatedOk; void unrelatedMismatch;",
        ]
      : [
          statement,
          "const scoped: RouteScopedGlobal = { marker: 'route-global' };",
          "const unrelated: UnrelatedPageGlobal = unrelatedPageValue;",
          `const numeric: ${broken ? "string" : "number"} = unrelated.value;`,
          "void scoped; void unrelated; void numeric;",
        ];
  return [
    '<script setup lang="ts">',
    ...body,
    "</script>",
    "<template><div /></template>",
    "",
  ].join("\n");
}

/** Integrate under the real-provider oracle after its exact source custody gate. */
export async function runPartitionControls(t: TestContext, corsaPath: string): Promise<void> {
  assert.equal(process.env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  const cli = process.env.VIZE_TEST_BIN;
  assert.ok(cli && path.isAbsolute(cli), "parent must supply its exact source-built CLI");
  assert.deepEqual(resolveVizeCommand(), [cli], "reject fallback to global/cargo CLI");
  assert.ok(
    path.isAbsolute(corsaPath) && fs.statSync(corsaPath).isFile(),
    "parent supplies actual Corsa",
  );
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-page-partition-secondary-"));
  const write = (relative: string, content: string) => {
    const target = path.join(workspace, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, content);
  };
  try {
    symlinkVueTypes(workspace);
    write(
      "package.json",
      json({ name: "secondary-page-partition", private: true, type: "module" }),
    );
    write("globals.d.ts", GLOBALS);
    write(
      "node_modules/vue-router/package.json",
      json({
        name: "vue-router",
        version: "0.0.0-synthetic",
        type: "module",
        exports: { "./auto-routes": { types: "./auto-routes.d.mts" } },
      }),
    );
    assert.ok(!GLOBALS.includes("auto-routes") && !GLOBALS.includes("RouteScopedGlobal"));
    assert.ok(Buffer.byteLength(SHARED) < 80, "bounded multiply imported shared leaf");

    function prepare(topology: Topology, statement: string, helper: boolean, broken = false) {
      write(HELPER, helper ? ROUTES : "export {};\n");
      fs.rmSync(path.join(workspace, "shared.ts"), { force: true });
      if (topology === "shared-leaf") write("shared.ts", SHARED);
      const sources: Record<string, string> = {};
      ROOTS.forEach((name, index) => {
        sources[name] = source(
          topology,
          index === 0 ? statement : `const root = ${index}; void root;`,
          broken,
        );
        write(name, sources[name]);
        assert.ok(
          !sources[name].includes("vue-router"),
          "authored bare import cannot confound the guard",
        );
      });
      return sources;
    }

    function configure(topology: Topology, enabled: boolean) {
      write(
        "tsconfig.json",
        json({
          compilerOptions: {
            target: "ES2022",
            lib: ["ES2022", "DOM"],
            module: "ESNext",
            moduleResolution: "bundler",
            rootDir: ".",
            strict: true,
            noEmit: true,
            skipLibCheck: true,
            types: [],
          },
          include: [...ROOTS, "globals.d.ts", ...(topology === "shared-leaf" ? ["shared.ts"] : [])],
          vueCompilerOptions: { plugins: enabled ? [PLUGIN] : [] },
        }),
      );
    }

    function paired(
      st: TestContext,
      label: string,
      topology: Topology,
      sources: Record<string, string>,
      imported: boolean,
      broken = false,
    ): Observation {
      const patterns = topology === "shared-leaf" ? [...ROOTS, "shared.ts"] : ROOTS;
      const observations = [1, 2].map((servers) => {
        assert.deepEqual(resolveVizeCommand(), [cli]);
        const result = runVizeCheck(workspace, corsaPath, [
          ...patterns,
          "--servers",
          String(servers),
          "--show-virtual-ts",
        ]) as Observation;
        assertReport(result, topology, sources, imported, broken);
        st.diagnostic(
          JSON.stringify({
            authority: "secondary-synthetic-no-provider-credit",
            label,
            topology,
            requestedServers: servers,
            workspace,
            sourceHashes: Object.fromEntries(
              [
                ...patterns,
                "globals.d.ts",
                HELPER,
                "tsconfig.json",
                "package.json",
                "node_modules/vue-router/package.json",
              ].map((file) => [file, sha256(fs.readFileSync(path.join(workspace, file), "utf8"))]),
            ),
            result,
          }),
        );
        return result;
      });
      assert.deepEqual(
        observations[1],
        observations[0],
        "full ordered report/status/stdout/stderr/virtual TS, same workspace",
      );
      return observations[0];
    }

    for (const topology of ["shared-leaf", "independent"] as const) {
      await t.test(
        `secondary ${topology}: implicit helper preserves whole-program globals`,
        (st) => {
          const sources = prepare(topology, BARE_CALL, true);
          configure(topology, false);
          const disabled = paired(st, "disabled-bare-call", topology, sources, false);
          configure(topology, true);
          const enabled = paired(st, "enabled-bare-call", topology, sources, true);
          assertOnlyInjectedCall(disabled, enabled);
          if (topology === "independent") {
            const broken = prepare(topology, BARE_CALL, true, true);
            paired(st, "unrelated-global-mismatch", topology, broken, true, true);
            prepare(topology, BARE_CALL, true);
            assert.deepEqual(
              paired(st, "repaired-unrelated-global", topology, sources, true),
              enabled,
            );
          }
          configure(topology, false);
          assert.deepEqual(paired(st, "disabled-repair", topology, sources, false), disabled);
        },
      );
      for (const helper of [true, false]) {
        for (const [name, statement] of NEGATIVES) {
          await t.test(
            `secondary ${topology}: ${name}, helper ${helper ? "present" : "absent"}`,
            (st) => {
              const sources = prepare(topology, statement, helper);
              configure(topology, false);
              const disabled = paired(st, "disabled-negative", topology, sources, false);
              configure(topology, true);
              const enabled = paired(st, "enabled-negative", topology, sources, false);
              assert.deepEqual(
                enabled,
                disabled,
                "complete configured no-edit result equals its own disabled baseline",
              );
            },
          );
        }
      }
    }
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
}
