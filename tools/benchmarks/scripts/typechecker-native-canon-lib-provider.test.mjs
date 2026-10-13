import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { qualifyCanonLibScope } from "./typechecker-native-canon-lib-scope.mjs";
import { physicalSource } from "./typechecker-native-canon-lib-source.mjs";
import {
  canonManifest,
  integration,
  source,
  append,
  manifestChange,
} from "./typechecker-native-canon-lib-law.mjs";
import { cargoConfigurations } from "./typechecker-native-canon-lib-config.mjs";

await test("locked derive providers and their transitive patch identities remain authoritative", () => {
  for (const name of ["serde", "serde_derive", "quote", "syn", "unicode-ident"]) {
    const changes = manifestChange("Cargo.toml", (m) => {
      m.patch["crates-io"] = { alias: { package: name, path: "vendor/alternate" } };
    });
    assert.throws(() => qualifyCanonLibScope(source(changes)), /patched derive provider/u);
  }
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source(
          manifestChange("Cargo.lock", (m) => {
            m.package.find((p) => p.name === "serde_derive").source =
              "git+https://example.invalid/serde?rev=0#0";
          }),
        ),
      ),
    /derive source/u,
  );
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source(
          manifestChange("Cargo.lock", (m) => {
            delete m.package.find((p) => p.name === "quote").checksum;
          }),
        ),
      ),
    /derive checksum/u,
  );
  // A crates.io update of the derive closure is an ordinary dependency change.
  assert.doesNotThrow(() =>
    qualifyCanonLibScope(
      source({
        ...manifestChange("Cargo.lock", (m) => {
          const derive = m.package.find((p) => p.name === "serde_derive");
          derive.version = "1.0.999";
          derive.checksum = "f".repeat(64);
        }),
        ...manifestChange("Cargo.toml", (m) => {
          m.workspace.dependencies.serde = { version: "=1.0.999", features: ["derive"] };
        }),
      }),
    ),
  );
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source(
          manifestChange("Cargo.lock", (m) => {
            m.package.push({ ...m.package.find((p) => p.name === "serde") });
          }),
        ),
      ),
    /ambiguous derive/u,
  );
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source({ ".cargo/config": '[source.crates-io]\nreplace-with = "unreviewed"\n' }),
      ),
    /source override/u,
  );
  const fake = "vendor/fake-provider/Cargo.toml";
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source({
          ...manifestChange("Cargo.toml", (m) => {
            m.patch["crates-io"] = { unrelated_alias: { path: "vendor/fake-provider" } };
          }),
          [fake]: '[package]\nname = "serde_derive"\nversion = "1.0.228"\n',
        }),
      ),
    /ambiguous patch package/u,
  );
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source(
          manifestChange("Cargo.toml", (m) => {
            m.replace = { "quote:1.0.45": { path: "vendor/fake-provider" } };
          }),
        ),
      ),
    /patched derive provider/u,
  );
  assert.throws(
    () =>
      qualifyCanonLibScope(
        source(
          manifestChange(canonManifest, (m) => {
            m["dev-dependencies"].serde = { package: "foreign", path: "../foreign" };
          }),
        ),
      ),
    /shadowed Canon serde/u,
  );
});

await test("physical symlink files and ancestor directories cannot supply authored registrations", () => {
  const temp = mkdtempSync(path.join(os.tmpdir(), "vize-canon-lib-source-"));
  try {
    mkdirSync(path.join(temp, "real"));
    writeFileSync(path.join(temp, "real", "case.rs"), "");
    symlinkSync(path.join(temp, "real", "case.rs"), path.join(temp, "alias.rs"));
    symlinkSync(path.join(temp, "real"), path.join(temp, "alias"));
    for (const file of ["alias.rs", "alias/case.rs"])
      assert.throws(() => physicalSource(temp).read(file), /symlink source/u);
  } finally {
    rmSync(temp, { recursive: true, force: true });
  }
});

for (const name of ["std", "core"]) {
  await test(`${name} cannot be replaced by a dependency alias providing derives`, () => {
    const changes = {
      ...manifestChange(canonManifest, (m) => {
        m.dependencies[name] = { package: "alternate", path: "../alternate" };
      }),
      ...append(integration, `use ${name}::generated as Debug; #[derive(Debug)] struct Carrier;`),
    };
    assert.throws(
      () => qualifyCanonLibScope(source(changes)),
      /shadowed standard derive provider/u,
    );
  });
}

for (const [label, text] of [
  ["paths", 'paths = ["vendor/alternate"]'],
  ["patch", '[patch.crates-io]\nserde_derive = { path = "vendor/alternate" }'],
  ["include", 'include = ["alternate.toml"]'],
  ["lock redirect", '[resolver]\nlockfile-path = "alternate.lock"'],
  ["compiler", '[build]\nrustc = "alternate"'],
  ["wrapper", '[build]\nrustc-wrapper = "alternate"'],
  ["workspace wrapper", '[build]\nrustc-workspace-wrapper = "alternate"'],
  ["runner", '[target.x86_64-unknown-linux-gnu]\nrunner = "alternate"'],
  ["environment", '[env]\nRUSTC_WRAPPER = "alternate"'],
]) {
  await test(`Cargo ${label} authority cannot change after the static census`, () => {
    assert.throws(() => qualifyCanonLibScope(source({ ".cargo/config": text })));
  });
}

await test("actual Cargo ancestor and home configuration metadata also rejects compiler overrides", () => {
  const temp = mkdtempSync(path.join(os.tmpdir(), "vize-canon-lib-config-"));
  try {
    mkdirSync(path.join(temp, "project"));
    mkdirSync(path.join(temp, ".cargo"));
    writeFileSync(
      path.join(temp, ".cargo", "config.toml"),
      '[build]\nrustc-wrapper = "alternate"\n',
    );
    const cargoContext = cargoConfigurations(path.join(temp, "project"), { HOME: temp });
    assert.equal(cargoContext.configurations.length, 1);
    assert.throws(() => qualifyCanonLibScope({ ...source(), cargoContext }), /compiler override/u);
  } finally {
    rmSync(temp, { recursive: true, force: true });
  }
});

await test("the authenticated unchanged Wild linker configuration remains admitted", () => {
  const runnerTemp = "/home/runner/_work/_temp";
  const cargoContext = {
    configurations: [
      {
        file: "/home/runner/.cargo/config.toml",
        text: `[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-Clink-arg=--ld-path=${runnerTemp}/wild-install/wild"]\n[target.x86_64-unknown-linux-musl]\nlinker = "clang"\nrustflags = ["-Clink-arg=--ld-path=${runnerTemp}/wild-install/wild"]\n`,
      },
    ],
    environment: [],
    runnerTemp,
  };
  qualifyCanonLibScope({ ...source(), cargoContext });
  const altered = {
    ...cargoContext,
    configurations: [
      {
        file: "/home/runner/.cargo/config.toml",
        text: '[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["--cfg", "unreviewed"]\n',
      },
    ],
  };
  assert.throws(
    () => qualifyCanonLibScope({ ...source(), cargoContext: altered }),
    /compiler flags/u,
  );
});
