import assert from "node:assert/strict";
import { test } from "node:test";
import {
  canonLibFilters,
  qualifyCanonLibCommands,
  qualifyCanonLibScope,
} from "./typechecker-native-canon-lib-scope.mjs";
import {
  actual,
  texts,
  canonManifest,
  integration,
  library,
  workflow,
  source,
  append,
  manifestChange,
} from "./typechecker-native-canon-lib-law.mjs";

await test("the authored current target closure contains both active original library cases and no outside matches", () => {
  const packet = qualifyCanonLibScope(actual);
  assert.deepEqual(qualifyCanonLibScope(source()).cases, packet.cases);
  assert.deepEqual(packet.originalCases, [
    `corsa_bridge::outside_import_editor_tests::${canonLibFilters[0]}`,
    `corsa_bridge::vue_document_package_config_tests::${canonLibFilters[1]}`,
  ]);
  assert.ok(packet.targets.some((target) => target.name === "davinci_projection_differential"));
  assert.ok(packet.cases.some((entry) => entry.name.includes("process_reuse::")));
  qualifyCanonLibCommands(workflow);
});

for (const filter of canonLibFilters) {
  await test(`an integration function matching ${filter} refuses library-only selection`, () => {
    assert.throws(
      () => qualifyCanonLibScope(source(append(integration, `#[test] fn ${filter}() {}`))),
      /outside-library selected/u,
    );
  });
  await test(`a module prefix or suffixed function matching ${filter} refuses selection`, () => {
    for (const text of [
      `mod ${filter} { #[test] fn unrelated() {} }`,
      `#[test] fn before_${filter}_after() {}`,
    ]) {
      assert.throws(
        () => qualifyCanonLibScope(source(append(integration, text))),
        /outside-library selected/u,
      );
    }
  });
  await test(`new Cargo file and directory targets matching ${filter} are discovered`, () => {
    for (const file of [
      "crates/vize_canon/tests/new_case.rs",
      "crates/vize_canon/tests/new_case/main.rs",
    ]) {
      assert.throws(
        () => qualifyCanonLibScope(source({ [file]: `#[test] fn ${filter}() {}` })),
        /outside-library selected/u,
      );
    }
  });
  await test(`future matching library cases remain selected by the unchanged ${filter} substring`, () => {
    qualifyCanonLibScope(source(append(library, `#[test] fn extra_${filter}_case() {}`)));
    qualifyCanonLibScope(source(append(library, `#[test] fn extra漢${filter}_case() {}`)));
  });
  await test(`raw identifiers and Unicode boundaries cannot truncate ${filter}`, () => {
    for (const text of [
      `#[test] fn r#${filter}() {}`,
      `#[test] fn before漢${filter}() {}`,
      `mod before漢${filter} { #[test] fn unrelated() {} }`,
    ]) {
      assert.throws(() => qualifyCanonLibScope(source(append(integration, text))));
    }
  });
}

await test("literal external and inline module paths cannot hide matching registrations", () => {
  const file = "crates/vize_canon/tests/selected_child.rs";
  const children = `#[test] fn ${canonLibFilters[0]}() {}`;
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source({
          ...append(integration, '#[path = "selected_child.rs"] mod child;'),
          [file]: children,
        }),
      ),
    /outside-library selected/u,
  );
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source({
          ...append(integration, "mod child;"),
          "crates/vize_canon/tests/child.rs": children,
        }),
      ),
    /outside-library selected/u,
  );
});

await test("platform branch union refuses a matching case even in an inactive windows branch", () => {
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source(append(integration, `#[cfg(windows)] #[test] fn ${canonLibFilters[0]}() {}`)),
      ),
    /outside-library selected/u,
  );
});

await test("Cargo-declared literal feature cfg retains ordinary integration targets", () => {
  const file = "crates/vize_canon/tests/declared_feature.rs";
  for (const feature of ["native", "future-registration"]) {
    const changes = {
      ...manifestChange(canonManifest, (m) => {
        m.features["future-registration"] = [];
      }),
      [file]: `#![cfg(feature = "${feature}")]\n#[test] fn ordinary_feature_case() {}`,
    };
    const packet = qualifyCanonLibScope(source(changes));
    assert.ok(packet.targets.some((target) => target.file === file));
    assert.ok(
      packet.cases.some((entry) => entry.file === file && entry.name === "ordinary_feature_case"),
    );
    assert.deepEqual(packet.originalCases, qualifyCanonLibScope(actual).originalCases);
  }
  for (const cfg of [
    'feature = "undeclared"',
    'not(feature = "undeclared")',
    'any(feature = "native", feature = "undeclared")',
  ])
    assert.throws(
      () => qualifyCanonLibScope(source(append(integration, `#[cfg(${cfg})] fn ordinary() {}`))),
      /undeclared Cargo cfg feature/u,
    );
});

await test("comments, raw literals and ordinary body edits do not freeze product source", () => {
  qualifyCanonLibScope(
    source(
      append(
        integration,
        `fn ordinary_body() { let _ = r#"#[test] fn ${canonLibFilters[0]}() {}"#; /* nested /* #[test] */ comment */ }`,
      ),
    ),
  );
  qualifyCanonLibScope(
    source({
      [library]: texts
        .get(library)
        .replace(
          "fn original_outside_import_preserves_types_in_native_editor() {",
          "fn original_outside_import_preserves_types_in_native_editor() {\n    let _ = 1 + 1;",
        ),
    }),
  );
});

for (const [label, text] of [
  ["item macro", "unknown_registration!();"],
  ["generator", "macro_rules! generated { () => { #[test] fn generated() {} } }"],
  ["include", 'include!("generated.rs");'],
  ["proc attribute", "#[foreign::test] fn generated() {}"],
  ["qualified test attribute", "#[test::generated] fn carrier() {}"],
  ["qualified ignore attribute", "#[ignore::generated] fn carrier() {}"],
  ["qualified expect attribute", "#[expect::generated] fn carrier() {}"],
  ["qualified path attribute", "#[path::generated] fn carrier() {}"],
  ["cfg_attr", "#[cfg_attr(unix, test)] fn generated() {}"],
  ["unknown cfg", '#[cfg(not(feature = "new-provider"))] #[test] fn generated() {}'],
  ["shadowed test", "use foreign::test;"],
  ["shadowed derive", "use foreign::Serialize;"],
  ["opaque wildcard", "use foreign::*;"],
  ["custom harness attribute", "#![test_runner(custom_runner)]"],
]) {
  await test(`${label} cannot establish a finite registration proof`, () => {
    assert.throws(() => qualifyCanonLibScope(source(append(integration, text))));
  });
}

await test("escaping, ambiguous and cyclic modules refuse the proof", () => {
  for (const changes of [
    append(integration, '#[path = "../../vize/src/main.rs"] mod escaped;'),
    {
      ...append(integration, "mod ambiguous;"),
      "crates/vize_canon/tests/ambiguous.rs": "",
      "crates/vize_canon/tests/ambiguous/mod.rs": "",
    },
    append(integration, '#[path = "ambient_declarations.rs"] mod cycle;'),
  ])
    assert.throws(() => qualifyCanonLibScope(source(changes)));
});

await test("default non-mod files retain their owned directory for inline literal paths", () => {
  const changes = {
    ...append(integration, "mod carrier;"),
    "crates/vize_canon/tests/carrier.rs": 'mod inline { #[path = "selected.rs"] mod nested; }',
    "crates/vize_canon/tests/carrier/inline/selected.rs": `#[test] fn ${canonLibFilters[0]}() {}`,
    "crates/vize_canon/tests/inline/selected.rs": "",
  };
  assert.throws(() => qualifyCanonLibScope(source(changes)), /outside-library selected/u);
});

await test("path-included files use mod-rs sibling semantics rather than a decoy stem directory", () => {
  const changes = {
    ...append(integration, '#[path = "peer/child.rs"] mod carrier;'),
    "crates/vize_canon/tests/peer/child.rs": "mod selected;",
    "crates/vize_canon/tests/peer/selected.rs": `#[test] fn ${canonLibFilters[1]}() {}`,
    "crates/vize_canon/tests/peer/child/selected.rs": "",
  };
  assert.throws(() => qualifyCanonLibScope(source(changes)), /outside-library selected/u);
});

await test("explicit targets, custom harnesses, build generators and library redirects remain closed", () => {
  const matching = "crates/vize_canon/tests/explicit/case.rs";
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source({
          ...manifestChange(canonManifest, (m) =>
            m.test.push({ name: "explicit", path: "tests/explicit/case.rs" }),
          ),
          [matching]: `#[test] fn ${canonLibFilters[1]}() {}`,
        }),
      ),
    /outside-library selected/u,
  );
  for (const mutate of [
    (m) => {
      m.test[0].harness = false;
    },
    (m) => {
      m.package.build = "build.rs";
    },
    (m) => {
      m.lib = { path: "redirect.rs" };
    },
    (m) => {
      m.features.default = [];
    },
  ])
    assert.throws(() => qualifyCanonLibScope(source(manifestChange(canonManifest, mutate))));
  assert.throws(
    () => qualifyCanonLibScope(source({ "crates/vize_canon/build.rs": "fn main() {}" })),
    /build script/u,
  );
  const binary = "crates/vize_canon/src/bin/future/main.rs";
  assert.throws(
    () => qualifyCanonLibScope(source({ [binary]: `#[test] fn ${canonLibFilters[0]}() {}` })),
    /outside-library selected/u,
  );
  const example = "crates/vize_canon/examples/future.rs";
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source({
          ...manifestChange(canonManifest, (m) => {
            m.example = [{ name: "future", test: true }];
          }),
          [example]: `#[test] fn ${canonLibFilters[0]}() {}`,
        }),
      ),
    /outside-library selected/u,
  );
});

await test("removed, ignored and conditionally disabled original library cases refuse admission", () => {
  for (const replacement of [
    "#[ignore] #[test]",
    "#[cfg(windows)] #[test]",
    "#[cfg_attr(unix, test)]",
  ]) {
    assert.throws(
      () =>
        qualifyCanonLibScope(
          source({ [library]: texts.get(library).replace("#[test]", replacement) }),
        ),
      /inactive original library case/u,
    );
  }
  assert.throws(() => qualifyCanonLibScope(source({ [library]: null })), /missing source/u);
  assert.throws(
    () => qualifyCanonLibScope(source({ [library]: `#![cfg(windows)]\n${texts.get(library)}` })),
    /inactive original library/u,
  );
});

for (const file of [
  "crates/vize_canon/src/lib.rs",
  "crates/vize_canon/src/corsa_bridge.rs",
  library,
  "crates/vize_canon/src/corsa_bridge/vue_document_package_config_tests.rs",
]) {
  await test(`original builtin test authority is retained through ${file}`, () => {
    for (const text of [
      "use foreign::test;",
      "use foreign::generated as test;",
      "unknown_ancestral_import!();",
      "#[foreign::attribute] fn unrelated() {}",
    ]) {
      assert.throws(
        () => qualifyCanonLibScope(source(append(file, text))),
        /registration authority|attribute or derive provider|ancestral attribute authority/u,
      );
    }
  });
}

await test("the two command contracts reject broadened filters, profile changes and lost receipt visibility", () => {
  for (const filter of canonLibFilters) {
    for (const changed of [
      workflow.replace(`--lib ${filter}`, `--lib prefix_${filter}`),
      workflow.replace(`--lib ${filter}`, `--lib ${filter} -- --exact`),
      workflow.replace(`--lib ${filter}`, filter),
    ])
      assert.throws(() => qualifyCanonLibCommands(changed));
  }
  assert.throws(() =>
    qualifyCanonLibCommands(
      workflow.replaceAll("profile.ci-opt.codegen-units=16", "profile.ci-opt.codegen-units=1"),
    ),
  );
  assert.throws(() => qualifyCanonLibCommands(workflow.replaceAll("-- --nocapture", "")));
});
