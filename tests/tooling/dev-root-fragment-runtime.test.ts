import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { parse } from "yaml";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { environment7502 } from "./support/native-attribute-values-7502-loader.ts";

interface Fixture {
  name: string;
  template: string;
  comments: boolean;
  whitespace: "condense" | "preserve";
  inherit: boolean;
  devRoot: boolean;
}
interface Capture {
  name: string;
  code: string;
}
const fixtures: Fixture[] = JSON.parse(
  readFileSync(
    new URL("../../crates/vize_atelier_dom/tests/fixtures/dev_root_fragment.json", import.meta.url),
    "utf8",
  ),
);
assert.equal(fixtures.length, 18);
assert.equal(new Set(fixtures.map((fixture) => fixture.name)).size, 18);
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const compiler = createRequire(ui.resolve("vue/package.json"))("@vue/compiler-sfc");
assert.equal(compiler.version, "3.5.35");
const mode = process.env.NODE_ENV === "production" ? "production" : "development";
process.env.NODE_ENV ??= mode;
const environment = await environment7502(mode);
const runtimeFixtures =
  mode === "production"
    ? fixtures.filter((fixture) => fixture.name === "production_strip")
    : fixtures;

function primary(fixture: Fixture): string {
  const result = compiler.compileTemplate({
    id: "dev-root-fragment",
    filename: "Child.vue",
    source: fixture.template,
    compilerOptions: { comments: fixture.comments, whitespace: fixture.whitespace },
  });
  assert.deepEqual(result.errors, []);
  assert.equal(result.code.includes("DEV_ROOT_FRAGMENT"), fixture.devRoot, fixture.name);
  return result.code;
}

async function execute(code: string) {
  // The template compiler's whole module remains intact; this supplies only
  // the surrounding component that an SFC script normally supplies.
  const { component, runtime } = await environment.load(
    code + "\nexport default {render}\n",
    "dom",
  );
  component.data = () => ({ ok: true, no: false, a: true, b: true });
  const warnings: string[] = [];
  const host = environment.window.document.createElement("div");
  const app = runtime.createApp(component, { class: "from-parent", id: "forwarded" });
  app.component("Inner", { render: () => runtime.h("div", { class: "child" }) });
  app.config.warnHandler = (message: string) => warnings.push(message);
  try {
    app.mount(host);
    const root = host.querySelector(".child");
    return {
      tree: environment.hostTree(host),
      warnings,
      class: root?.getAttribute("class") ?? null,
      id: root?.getAttribute("id") ?? null,
    };
  } finally {
    app.unmount();
  }
}

function assertInheritance(fixture: Fixture, observed: Awaited<ReturnType<typeof execute>>) {
  assert.equal(observed.id === "forwarded", fixture.inherit, fixture.name);
  if (fixture.inherit) {
    assert.equal(observed.class, "child from-parent", fixture.name);
    assert.deepEqual(observed.warnings, [], fixture.name);
  }
}

test("Vue 3.5 root policy and runtime oracle cover comments, true siblings and production", async () => {
  for (const fixture of runtimeFixtures)
    assertInheritance(fixture, await execute(primary(fixture)));
});

test(
  "the actual development runtime refuses the old stable-only root fragment",
  {
    skip: mode === "production" ? "DEV_ROOT_FRAGMENT is a development runtime policy" : false,
  },
  async () => {
    const fixture = fixtures.find((fixture) => fixture.name === "reporter_newline")!;
    const original = primary(fixture);
    const marker = "2112 /* STABLE_FRAGMENT, DEV_ROOT_FRAGMENT */";
    assert.equal(original.split(marker).length, 2);
    const broken = await execute(original.replace(marker, "64 /* STABLE_FRAGMENT */"));
    assert.equal(broken.class, "child");
    assert.equal(broken.id, null);
    if (mode === "development")
      assert(
        broken.warnings.some((warning) => warning.includes("Extraneous non-props attributes")),
      );
    assert.throws(() => assertInheritance(fixture, broken));
  },
);

const capturePath = process.env.VIZE_DEV_ROOT_FRAGMENT_CAPTURE;
const required = process.env.VIZE_DEV_ROOT_FRAGMENT_REQUIRE_CAPTURE === "1";
test(
  "fresh source compiler modules match primary DOM and forward class/id without warnings",
  {
    skip:
      !capturePath && !required
        ? "source compiler capture is supplied by the native Actions lane"
        : false,
  },
  async () => {
    assert(capturePath, "a required source compiler capture must be provided");
    const captures: Capture[] = JSON.parse(readFileSync(capturePath, "utf8"));
    assert.deepEqual(
      captures.map((capture) => capture.name),
      fixtures.map((fixture) => fixture.name),
    );
    for (const fixture of runtimeFixtures) {
      const capture = captures.find((entry) => entry.name === fixture.name);
      assert(capture);
      const code = capture.code;
      assert.equal(typeof code, "string");
      assert.equal(code.includes("DEV_ROOT_FRAGMENT"), fixture.devRoot, fixture.name);
      const actual = await execute(code);
      assertInheritance(fixture, actual);
      assert.deepEqual(actual, await execute(primary(fixture)), fixture.name);
    }
  },
);

test("source changes and runtime witnesses require the executable native carrier", () => {
  for (const path of [
    "davinci/vize_l1_to_l2/src/emit/fragment.rs",
    "crates/vize_armature/src/parser/whitespace.rs",
    "crates/vize_armature/src/parser/whitespace/tests.rs",
    "crates/vize_atelier_dom/tests/dev_root_fragment.rs",
    "crates/vize_atelier_dom/tests/fixtures/dev_root_fragment.json",
    "tests/tooling/dev-root-fragment-runtime.test.ts",
  ])
    assert(planNativeRuntime([path]).matrix.lane.includes("selected-template"), path);
  const action = parse(
    readFileSync(
      new URL("../../.github/actions/test-native-selected-template/action.yml", import.meta.url),
      "utf8",
    ),
  );
  const step = action.runs.steps.find(
    (candidate: { env?: Record<string, string> }) =>
      candidate.env?.VIZE_DEV_ROOT_FRAGMENT_REQUIRE_CAPTURE === "1",
  );
  assert(step, "the source runtime proof is mandatory, not an optional local skip");
  assert.equal(
    step.env.VIZE_DEV_ROOT_FRAGMENT_CAPTURE,
    "${{ runner.temp }}/dev-root-fragment.json",
  );
  assert.deepEqual(step.run.trim().split("\n"), [
    "cargo test -p vize_atelier_dom --profile ci --test dev_root_fragment",
    "NODE_ENV=development vp node --test tests/tooling/dev-root-fragment-runtime.test.ts",
    "NODE_ENV=production vp node --test tests/tooling/dev-root-fragment-runtime.test.ts",
  ]);
});
