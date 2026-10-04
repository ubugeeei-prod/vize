import assert from "node:assert/strict";
import { test } from "node:test";
import path from "node:path";

import { repoRoot, withPinnedFixtureWorkspace } from "../../_helpers/realworld-patch.ts";
import {
  resolveTsgoBinary,
  resolveVizeCommand,
  resolveVueTscBinary,
} from "../../_helpers/realworld-typecheck.ts";
import {
  configure,
  control,
  type ErrorSpec,
  PAGE_PATH,
  PLAYGROUND,
  prepareProvider,
  ROUTER_REVISION,
  ROUTES_PATH,
  sha256,
  typeofReference,
  TYPEOF_REVISION,
} from "../../fixtures/typechecker/page-route-types/support.ts";
import { resolveVizeLaunchCommand } from "../../tooling/support/lsp/launch.ts";
import { runPartitionControls } from "../../fixtures/typechecker/page-route-types/partition-controls.ts";
import { editorCycleFor } from "../../fixtures/typechecker/page-route-types/editor-cycle.ts";
import { pageRouteObserver } from "../../fixtures/typechecker/page-route-types/cli-observer.ts";

const numericMethodError = /^Property 'toUpperCase' does not exist on type 'number'\.$/;
const fallbackError: ErrorSpec = { code: 2339, needle: "route.params.userId", token: "userId" };

function replaceOne(source: string, before: string, after: string): string {
  assert.equal(source.split(before).length, 2, `unique patch anchor: ${before}`);
  return source.replace(before, after);
}

await test("page-scoped Vue Router types preserve complete CLI/editor diagnostics and authored ranges", async (t) => {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();
  assert.equal(
    process.env.VIZE_TEST_REQUIRE_TSGO,
    "1",
    "this oracle requires the actual native backend",
  );
  assert.equal(
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD,
    "1",
    "require source-built CLI/editor Actions custody",
  );
  assert.ok(
    process.env.VIZE_TEST_BIN,
    "use the exact source-built CLI, without global binary fallback",
  );
  const verified = resolveVizeLaunchCommand(undefined, process.env.VIZE_TEST_BIN, {
    required: true,
    repoRoot,
  });
  assert.deepEqual(resolveVizeCommand(), [verified[0]], "CLI uses the same verified source binary");
  assert.equal(path.resolve(repoRoot, process.env.VIZE_LSP_BIN ?? ""), verified[0]);

  await runPartitionControls(t, corsaPath);

  await withPinnedFixtureWorkspace(
    { fixtureId: "vue-router", includePaths: [PAGE_PATH, ROUTES_PATH] },
    async (fixture) => {
      const provider = prepareProvider(fixture);
      const providerEvidence = provider.evidence;
      const upstreamSource = fixture.read(PAGE_PATH);
      const upstreamRoutes = fixture.read(ROUTES_PATH);
      t.diagnostic(JSON.stringify({ routerSource: ROUTER_REVISION, providerEvidence }));

      const observe = pageRouteObserver(
        t,
        fixture,
        (observation) => provider.record(observation),
        verified[0],
        corsaPath,
        vueTscPath,
        upstreamSource,
        upstreamRoutes,
        providerEvidence.archive.archiveSha256,
      );

      const editorCycle = editorCycleFor(fixture, observe);

      await t.test(
        "real generated map and numeric upstream page, compiler rootDir, and mapped edits",
        async () => {
          configure(fixture, corsaPath);
          const original = observe(upstreamSource, []);
          assert.deepEqual(observe(upstreamSource, []).report, original.report);
          let clean = replaceOne(
            upstreamSource,
            "const route = useRoute()",
            "const route = useRoute(); const typedId: number = route.params.userId",
          );
          clean = replaceOne(clean, "// id: 'int',", "userId: 'int',");
          clean = replaceOne(clean, "$route.path", "$route.params.userId.toFixed()");
          await editorCycle(
            clean,
            [
              {
                name: "same-line script diagnostic after injected type argument",
                source: replaceOne(clean, "typedId: number", "typedId: string"),
                errors: [
                  {
                    code: 2322,
                    needle: "const typedId: string",
                    token: "typedId",
                    message: /^Type 'number' is not assignable to type 'string'\.$/,
                  },
                ],
              },
              {
                name: "template $route keeps the numeric parser type",
                source: replaceOne(
                  clean,
                  "$route.params.userId.toFixed()",
                  "$route.params.userId.toUpperCase()",
                ),
                errors: [
                  {
                    code: 2339,
                    needle: "$route.params.userId.toUpperCase()",
                    token: "toUpperCase",
                    message: numericMethodError,
                  },
                ],
              },
              {
                name: "definePage restricts path keys to this generated file entry",
                source: replaceOne(clean, "userId: 'int',", "unknownId: 'int',"),
                errors: [
                  {
                    code: 2353,
                    needle: "unknownId: 'int'",
                    token: "unknownId",
                    message: /'unknownId' does not exist/,
                    renderCase: "define-page-unknown-id",
                  },
                ],
              },
            ],
            PAGE_PATH,
            (source) => source,
            "typedId",
          );
        },
      );

      await t.test(
        "object plugin rootDir overrides compiler rootDir; inherited compiler rootDir works",
        () => {
          for (const options of [
            { compilerRoot: ".", pluginRoot: PLAYGROUND },
            { inherited: true },
          ]) {
            configure(fixture, corsaPath, PAGE_PATH, options);
            observe(upstreamSource, []);
          }
        },
      );

      await t.test(
        "valid TypeScript-only syntax and decoded identifiers keep page specialization",
        async () => {
          configure(fixture, corsaPath);
          const syntax = replaceOne(
            upstreamSource,
            "const route = useRoute()",
            "const identity = <T>(value: T) => value; const asserted = <number>1; const route = useRoute(); const typedId: number = identity(route.params.userId) + asserted",
          );
          observe(syntax, []);
          const escaped = syntax.replaceAll("useRoute", "useR\\u006fute");
          await editorCycle(escaped, [
            {
              name: "diagnostic after a decoded escaped useRoute and TypeScript-only expressions",
              source: replaceOne(escaped, "typedId: number", "typedId: string"),
              errors: [
                {
                  code: 2322,
                  needle: "const typedId: string",
                  token: "typedId",
                  message: /^Type 'number' is not assignable to type 'string'\.$/,
                },
              ],
            },
          ]);
        },
      );

      await t.test(
        "explicit route argument, type argument, and definePage type argument stay explicit",
        () => {
          configure(fixture, corsaPath);
          observe(control("explicit.vue"), []);
          observe(control("explicit-other.vue"), [
            { code: 2339, needle: "argument.params.userId", token: "userId" },
            { code: 2339, needle: "generic.params.userId", token: "userId" },
          ]);
        },
      );

      await t.test(
        "JavaScript setup uses the numeric return type without authored TypeScript syntax",
        async () => {
          configure(fixture, corsaPath);
          const clean = control("javascript.vue");
          await editorCycle(clean, [
            {
              name: "JavaScript diagnostic after wrapped useRoute()",
              source: replaceOne(
                clean,
                "const formatted = route.params.userId.toFixed()",
                "const formatted = route.params.userId.toUpperCase()",
              ),
              errors: [
                {
                  code: 2339,
                  needle: "route.params.userId.toUpperCase()",
                  token: "toUpperCase",
                  message: numericMethodError,
                },
              ],
            },
          ]);
        },
      );

      await t.test(
        `typeof arm compares published ${TYPEOF_REVISION}, not the older fixture plugin`,
        async () => {
          configure(fixture, corsaPath);
          const clean = control("typeof.vue");
          await editorCycle(
            clean,
            [
              {
                name: "typeof useRoute has the same page-scoped numeric type",
                source: replaceOne(
                  clean,
                  "return route.params.userId",
                  "return route.params.userId.toUpperCase()",
                ),
                errors: [
                  {
                    code: 2339,
                    needle: "route.params.userId.toUpperCase()",
                    token: "toUpperCase",
                    message: numericMethodError,
                  },
                ],
              },
            ],
            PAGE_PATH,
            typeofReference,
          );
        },
      );

      await t.test("disabled plugin and an unmapped component preserve all-route fallback", () => {
        configure(fixture, corsaPath, PAGE_PATH, { enabled: false });
        observe(control("fallback.vue"), [fallbackError]);
        const unmapped = `${PLAYGROUND}/src/components/Unmapped.vue`;
        configure(fixture, corsaPath, unmapped);
        observe(control("fallback.vue"), [fallbackError], unmapped);
      });

      await t.test("normal script stays outside script-setup adaptation", () => {
        configure(fixture, corsaPath);
        observe(control("normal-script.vue"), [fallbackError]);
      });

      await t.test("members, aliases, strings, comments and local shadows stay unchanged", () => {
        configure(fixture, corsaPath);
        // The upstream identifier-only visitor rewrites local shadows. This
        // compatibility guard therefore compares its disabled reference.
        observe(control("local-shadows.vue"), [], PAGE_PATH, (source) => source, {
          enabled: false,
        });
      });

      await t.test(
        "Vize project-root fallback uses an explicit control map, not feed-plugin fallback",
        () => {
          const product = { compilerRoot: false, projectRootMap: true } as const;
          configure(fixture, corsaPath, PAGE_PATH, product);
          observe(
            replaceOne(control("explicit.vue"), 'useRoute("/users/[userId=int]")', "useRoute()"),
            [],
            PAGE_PATH,
            (source) => source,
            { ...product, pluginRoot: "." },
            product,
          );
        },
      );

      assert.equal(
        fixture.read(ROUTES_PATH),
        upstreamRoutes,
        "never replace the real generated primary map",
      );
      assert.equal(sha256(fixture.read(ROUTES_PATH)), providerEvidence.generatedRoutes);
    },
  );
});
