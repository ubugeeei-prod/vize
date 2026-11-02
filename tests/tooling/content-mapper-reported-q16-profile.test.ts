import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import test from "node:test";
import {
  q16Package,
  q16Profile,
  verifyQ16ProviderNode,
  verifyQ16Lock,
  verifyQ16RequiredEnvironment,
  q16Cases,
  selectQ16Cases,
  verifyQ16List,
} from "../../tools/support/compat/content-mapper/reported-q16.ts";
const workflow = fs.readFileSync(
  new URL("../../.github/workflows/content-mapper-conformance.yml", import.meta.url),
  "utf8",
);
test("11373 is a closed independent report profile while the entire previous workflow is restored", () => {
  const before = workflow.replace(
    'options: [all, A, A-with-declaration, B, C, D, "11373"]',
    "options: [all, A, A-with-declaration, B, C, D]",
  );
  assert.equal(
    createHash("sha256").update(before).digest("hex"),
    "4c95582bc43334ab1c742b5a26841a610806e398454123e8ff6c600d6c67ee31",
  );
  for (const key of ["all", "A", "A-with-declaration", "B", "C", "D"]) {
    assert.deepEqual(q16Profile(key).package, q16Package);
    verifyQ16ProviderNode("v26.13.0", key);
    assert.throws(() => verifyQ16ProviderNode("v26.12.1", key));
  }
  const profile = q16Profile("11373");
  assert.deepEqual(profile.package.dependencies, {
    tsrs: "0.20.1-ts7.1.0-beta",
    typescript: "5.9.3",
    vue: "3.5.43",
    "vue-tsc": "3.2.6",
  });
  assert.deepEqual(profile.reported, {
    vize: "0.640.0",
    node: "26.12.1",
    platform: "macOS",
    lock: "not supplied",
  });
  verifyQ16ProviderNode("v26.12.1", "11373");
  assert.throws(() => verifyQ16ProviderNode("v26.13.0", "11373"));
  for (const value of [undefined, "", "11373,B", "latest"]) assert.throws(() => q16Profile(value));
  const lock = {
    lockfileVersion: 3,
    packages: { "": { dependencies: profile.package.dependencies } },
  };
  verifyQ16Lock(lock, profile.package.dependencies);
  assert.throws(() => verifyQ16Lock(lock));
  assert.throws(() =>
    verifyQ16Lock(
      { ...lock, packages: { "": { dependencies: q16Package.dependencies } } },
      profile.package.dependencies,
    ),
  );
  const e: NodeJS.ProcessEnv = {
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_TEST_REQUIRE_GLOBAL_COMPONENT_LISTENER: "1",
    CORSA_PATH: "/owned/tsrs",
    VIZE_TEST_CONTENT_MAPPER_TSGO: "/owned/tsrs",
    VIZE_TEST_CONTENT_MAPPER_VUE: "/owned/vue",
    VIZE_REPORTED_GLOBAL_COMPONENT_LISTENER_CAPTURE: "/owned/capture",
  };
  verifyQ16RequiredEnvironment(e, "11373");
  for (const key of Object.keys(e))
    assert.throws(() => verifyQ16RequiredEnvironment({ ...e, [key]: undefined }, "11373"));
  assert.throws(() =>
    verifyQ16RequiredEnvironment(
      { ...e, VIZE_TEST_REQUIRE_GLOBAL_COMPONENT_LISTENER: "0" },
      "11373",
    ),
  );
  assert.throws(() => verifyQ16RequiredEnvironment({ ...e, VIZE_TEST_DISABLE_TSGO: "1" }, "11373"));
  const only = selectQ16Cases("11373");
  assert.deepEqual(only, [
    [
      "global-component-listener",
      "content_mapper_tsgo_cli::global_component_listener_11373::reported_components_and_typed_controls_keep_real_diagnostics",
    ],
  ]);
  verifyQ16List(only[0][1] + ": test", only);
  assert.throws(() => verifyQ16List(q16Cases.map((r) => r[1] + ": test").join("\n"), only));
  const runner = fs.readFileSync(
    new URL("../../tools/support/compat/content-mapper/reported-q16.ts", import.meta.url),
    "utf8",
  );
  assert(
    runner.includes(
      "fs.appendFileSync(process.env.GITHUB_ENV!, \x60Q16_PROVIDER_NODE=${profile.reported.node}\\n\x60)".replaceAll(
        "\x60",
        "`",
      ),
    ),
  );
  assert(runner.includes("verifyQ16Lock(lock, profile.package.dependencies)"));
});
