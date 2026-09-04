import assert from "node:assert/strict";
import { test } from "node:test";

import {
  createCompatibilityContext,
  readCompatibilityLedger,
  validateCompatibilityLedger,
} from "../../legacy-tools/fixtures/fixture-compatibility-ledger.mjs";
import {
  createCompatibilityReport,
  formatCompatibilityReport,
} from "../../legacy-tools/fixtures/fixture-compatibility-report.mjs";
import { capabilityCounts as counts } from "./support/fixture-compatibility-counts.ts";

const context = createCompatibilityContext();
const ledger = readCompatibilityLedger();

test("compatibility ledger joins every fixture inventory exactly once", () => {
  const validated = validateCompatibilityLedger(ledger, context);
  assert.equal(validated.fixtureMap.size, 146);
  assert.equal(
    [...validated.fixtureMap.values()].filter((fixture) =>
      fixture.memberships.includes("ecosystem"),
    ).length,
    142,
  );
  assert.equal(
    [...validated.fixtureMap.values()].filter((fixture) => fixture.memberships.includes("app"))
      .length,
    16,
  );
  assert.deepEqual(
    [...validated.fixtureMap.values()]
      .filter(
        (fixture) =>
          fixture.memberships.includes("app") && !fixture.memberships.includes("ecosystem"),
      )
      .map((fixture) => fixture.fixturePath),
    [
      "tests/_fixtures/_git/frontend-phpcon-do-website",
      "tests/_fixtures/_git/npmx.dev",
      "tests/_fixtures/_git/nuxt-ui",
      "tests/_fixtures/_git/vuefes-2025",
    ],
  );
});

test("report keeps present, exercised, and runtime evidence separate", () => {
  const report = createCompatibilityReport(ledger, context);
  assert.deepEqual(report.inventories, {
    gitlinks: 146,
    ecosystem: 142,
    app: 16,
    appOnly: 4,
  });
  assert.deepEqual(
    Object.fromEntries(
      Object.entries(report.oracles).map(([kind, oracle]) => [kind, oracle.fixtureCount]),
    ),
    {
      compiler: 142,
      "formatter-idempotency": 142,
      linter: 142,
      typechecker: 142,
      "production-build": 4,
      "authored-lsp": 23,
      "vue-tsc-parity": 11,
      ssr: 3,
      hydration: 4,
      preview: 0,
      vrt: 5,
      "real-vite-hmr": 2,
    },
  );
  assert.deepEqual(report.capabilities["vue-generation"], {
    "0.x": counts(0, 0, 0, []),
    "1.x": counts(0, 0, 0, []),
    "2.x": counts(3, 3, 0, [
      "tests/_fixtures/_git/mobile-web-best-practice",
      "tests/_fixtures/_git/vue-element-admin",
      "tests/_fixtures/_git/vue2-elm",
    ]),
    "2.7": counts(0, 1, 0, ["tests/_fixtures/_git/vue-element-admin"]),
    "3.x": counts(1, 1, 1, ["tests/_fixtures/_git/npmx.dev"]),
  });
  assert.deepEqual(report.capabilities["api-style"], {
    "options-api": counts(1, 1, 0, ["tests/_fixtures/_git/vue-element-admin"]),
    "class-api": counts(1, 1, 0, ["tests/_fixtures/_git/mobile-web-best-practice"]),
    "composition-api": counts(1, 1, 0, ["tests/_fixtures/_git/create-vue"]),
    "script-setup": counts(1, 1, 0, ["tests/_fixtures/_git/create-vue"]),
  });
  for (const macro of ["define-page-meta", "use-head", "use-seo-meta"]) {
    assert.deepEqual(
      report.capabilities["nuxt-macro"][macro],
      counts(1, 1, 1, ["tests/_fixtures/_git/npmx.dev"]),
    );
  }

  const presentOnly = structuredClone(ledger);
  const vue3 = presentOnly.capabilities.find(
    (capability) => capability.dimension === "vue-generation" && capability.value === "3.x",
  );
  assert.ok(vue3);
  vue3.levels = ["present"];
  const presentOnlyReport = createCompatibilityReport(presentOnly, context);
  assert.deepEqual(
    presentOnlyReport.capabilities["vue-generation"]["3.x"],
    counts(1, 0, 0, ["tests/_fixtures/_git/npmx.dev"]),
    "source presence must not be promoted to an exercised or runtime oracle",
  );

  const reordered = structuredClone(ledger);
  reordered.capabilities.reverse();
  reordered.oracles.reverse();
  reordered.unresolved.reverse();
  assert.equal(
    formatCompatibilityReport(createCompatibilityReport(reordered, context)),
    formatCompatibilityReport(report),
    "the report must be byte-identical under reordered ledger input",
  );
});

test("invalid ledger mutations fail closed", () => {
  for (const [name, mutate, message] of mutationCases()) {
    const mutated = structuredClone(ledger);
    mutate(mutated);
    assert.throws(() => validateCompatibilityLedger(mutated, context), message, name);
  }
});

function mutationCases(): Array<[string, (value: typeof ledger) => void, RegExp]> {
  return [
    ["schema", (value) => (value.schema = "other"), /unsupported ledger schema/],
    ["version", (value) => (value.version = 2), /unsupported ledger version/],
    ["unknown root field", (value) => (value.extra = true), /shape is not closed/],
    [
      "unknown tracking issue field",
      (value) => (value.trackingIssues.extra = 1),
      /shape is not closed/,
    ],
    ["unknown fixture field", (value) => (value.fixtures[0].extra = true), /shape is not closed/],
    ["missing fixture", (value) => value.fixtures.pop(), /exactly partition/],
    [
      "duplicate fixture",
      (value) => value.fixtures.push(structuredClone(value.fixtures.at(-1))),
      /duplicates/,
    ],
    ["unsorted fixture paths", (value) => value.fixtures.reverse(), /codepoint sorted/],
    [
      "duplicate capability claim",
      (value) => value.capabilities.push(structuredClone(value.capabilities[0])),
      /capability claims contain duplicates/,
    ],
    [
      "membership drift",
      (value) =>
        value.fixtures.find((fixture) => fixture.memberships.length === 1).memberships.push("app"),
      /membership drifted/,
    ],
    [
      "unknown membership",
      (value) => (value.fixtures[0].memberships[0] = "preview"),
      /unknown membership/,
    ],
    [
      "unknown capability dimension",
      (value) => (value.capabilities[0].dimension = "framework"),
      /unknown capability dimension/,
    ],
    [
      "unknown capability value",
      (value) => (value.capabilities[0].value = "4.x"),
      /unknown vue-generation value/,
    ],
    [
      "unknown capability level",
      (value) => (value.capabilities[0].levels[0] = "inferred"),
      /unknown capability level/,
    ],
    [
      "unknown capability evidence field",
      (value) => (value.capabilities[0].evidence.extra = true),
      /shape is not closed/,
    ],
    [
      "runtime capability without runtime oracle",
      (value) => {
        const capability = value.capabilities.find(
          (candidate) =>
            candidate.fixturePath === "tests/_fixtures/_git/create-vue" &&
            candidate.value === "composition-api",
        );
        capability.fixturePath = "tests/_fixtures/_git/vuefes-2025";
        capability.levels.push("runtime");
      },
      /runtime capability lacks matching runtime oracle evidence/,
    ],
    [
      "runtime capability without exercised level",
      (value) => {
        value.capabilities.find((capability) => capability.levels.includes("runtime")).levels = [
          "present",
          "runtime",
        ];
      },
      /runtime capability must also be present and exercised/,
    ],
    [
      "runtime capability without present level",
      (value) => {
        value.capabilities.find((capability) => capability.levels.includes("runtime")).levels = [
          "exercised",
          "runtime",
        ];
      },
      /runtime capability must also be present and exercised/,
    ],
    [
      "stale evidence path",
      (value) => (value.capabilities[0].evidence.file = "tests/missing.ts"),
      /evidence file does not exist/,
    ],
    [
      "stale evidence selector",
      (value) => (value.capabilities[0].evidence.selector = "missing exact selector"),
      /evidence selector is stale/,
    ],
    [
      "ambiguous evidence selector",
      (value) => (value.capabilities[0].evidence.selector = "test("),
      /evidence selector is stale/,
    ],
    [
      "runtime capability claim outside App membership",
      (value) => {
        value.capabilities.find((capability) => capability.levels.includes("runtime")).fixturePath =
          "tests/_fixtures/_git/airi";
      },
      /runtime capability claim is not an App fixture/,
    ],
    [
      "unknown oracle kind",
      (value) => (value.oracles[0].kind = "source-present"),
      /unknown oracle kind/,
    ],
    ["unknown oracle field", (value) => (value.oracles[0].extra = true), /shape is not closed/],
    [
      "unknown oracle selection field",
      (value) => (value.oracles[0].selection.extra = true),
      /shape is not closed/,
    ],
    [
      "unknown oracle selection type",
      (value) => (value.oracles[0].selection = { type: "all" }),
      /unknown oracle selection type/,
    ],
    [
      "duplicate oracle",
      (value) => value.oracles.push(structuredClone(value.oracles[0])),
      /oracle fixture claims contain duplicates/,
    ],
    [
      "unknown selected fixture",
      (value) => {
        value.oracles.find(
          (oracle) => oracle.selection.type === "fixtures",
        ).selection.fixturePaths[0] = "tests/_fixtures/_git/not-registered";
      },
      /selected unknown fixture/,
    ],
    [
      "runtime oracle outside App membership",
      (value) => {
        value.oracles.find((oracle) => oracle.kind === "ssr").selection.fixturePaths[0] =
          "tests/_fixtures/_git/airi";
      },
      /App oracle is not an App fixture/,
    ],
    [
      "unknown unresolved field",
      (value) => (value.unresolved[0].extra = true),
      /shape is not closed/,
    ],
    [
      "unknown unresolved dimension",
      (value) => (value.unresolved[0].dimension = "other"),
      /unknown unresolved dimension/,
    ],
    [
      "unknown unresolved state",
      (value) => (value.unresolved[0].state = "ignored"),
      /unknown unresolved state/,
    ],
    [
      "missing unresolved reason",
      (value) => (value.unresolved[0].reason = ""),
      /unresolved reason must be a non-empty string/,
    ],
    [
      "unknown unresolved value",
      (value) => (value.unresolved[0].value = "4.x"),
      /unknown vue-generation unresolved value/,
    ],
    [
      "missing tracking issue",
      (value) => (value.unresolved[0].trackingIssue = 0),
      /trackingIssue must be a positive integer/,
    ],
    [
      "required unresolved state drift",
      (value) => (value.unresolved[0].state = "excluded"),
      /unresolved state drifted/,
    ],
    [
      "required unresolved owner drift",
      (value) => (value.unresolved[0].trackingIssue = 1),
      /unresolved owner drifted/,
    ],
    [
      "silent legacy omission",
      (value) => value.unresolved.splice(0, 1),
      /missing explicit unresolved dimension/,
    ],
    [
      "silent remaining corpus omission",
      (value) => {
        const index = value.unresolved.findIndex(
          (item) =>
            item.dimension === "corpus-capability-classification" &&
            item.value === "remaining-gitlinks",
        );
        value.unresolved.splice(index, 1);
      },
      /missing explicit unresolved dimension/,
    ],
    [
      "silent preview omission",
      (value) => {
        const index = value.unresolved.findIndex(
          (item) => item.dimension === "oracle-coverage" && item.value === "preview",
        );
        value.unresolved.splice(index, 1);
      },
      /missing explicit unresolved dimension/,
    ],
  ];
}
