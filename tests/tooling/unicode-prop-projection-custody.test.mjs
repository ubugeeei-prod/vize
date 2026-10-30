import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  certifyUnaffectedProjectionPackets,
  isProjectionSourcePath,
} from "../../tools/support/compat/github/unicode-prop-projection-custody.mjs";
import {
  canonicalPaths,
  nativeAttributesPath,
} from "../../tools/support/compat/content-mapper/constructor-projection-parity.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const importPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/content-mapper-type-imports.d.ts.txt";
const assets = [...canonicalPaths, nativeAttributesPath, importPath];
const encode = (value) => Buffer.from(JSON.stringify(value));
const bytes = (value) => Buffer.byteLength(value);

function packet(target, current, marker) {
  const source = "<template>日本 🧭</template>";
  const body = "const value = '日本 🧭';\n";
  const declaration = current
    ? "export type { $Slots as Slots };\n"
    : "export type Slots = $Slots;\n";
  const slotMarker = marker ? `type Marker = Partial<${current ? "$Slots" : "Slots"}>;\n` : "";
  const tail = "const tail = value;\n";
  const text = body + declaration + slotMarker + tail;
  const tailStart = bytes(body + declaration + slotMarker);
  const diagnostic = { start: 0, length: 1, code: 100001, messageText: "original diagnostic" };
  if (target === "content-mapper") {
    return {
      text,
      extension: ".tsx",
      mappings: [[0, bytes(body), 0, bytes(source), 1, 1048575]],
      semanticLinks: [
        {
          sourceStart: 0,
          sourceLength: 5,
          targetStart: tailStart,
          targetLength: 5,
          kind: "original",
        },
      ],
      diagnosticDirectives: {
        unusedExpectDirectiveDiagnostics: [],
        directives: [[0, 1, 0, bytes(body), 0, 0]],
      },
      diagnostics: [diagnostic],
    };
  }
  return {
    virtual_ts: text,
    virtual_ts_mappings: [
      {
        generated: { start: 0, end: bytes(body) },
        authored: { start: 0, end: bytes(source) },
        subSpans: [],
      },
    ],
    virtual_ts_semantic_links: [
      {
        source: { start: 0, end: 5 },
        target: { start: tailStart, end: tailStart + 5 },
        kind: "original",
      },
    ],
    diagnostics: [diagnostic],
    error_count: 1,
  };
}

function fixture() {
  const base = [],
    head = [];
  for (const newline of ["\n", "\r\n"]) {
    for (const name of ["generic", "invalid", "repaired", "plain", "javascript", "tsx"]) {
      for (const target of ["vue3", "options-api", "legacy-vue2", "content-mapper"]) {
        for (const options of target === "content-mapper"
          ? Array.from({ length: 16 }, (_, i) => i)
          : [undefined]) {
          const row = {
            case: name,
            newline,
            source: "<template>日本 🧭</template>",
            target,
            ...(options === undefined ? {} : { options }),
          };
          base.push({ ...row, packet: packet(target, false, name === "plain") });
          head.push({
            ...row,
            packet: packet(target, name === "plain" || name === "javascript", name === "plain"),
          });
        }
      }
    }
  }
  return { base, head };
}

function replay(base, head, baseRoot = root, headRoot = root) {
  return certifyUnaffectedProjectionPackets(encode(base), encode(head), baseRoot, headRoot);
}

const mapperIndex = (rows) =>
  rows.findIndex((row) => row.case === "plain" && row.target === "content-mapper");

test("Unicode consumer retains all 228 packets under the existing finite certificate", () => {
  const { base, head } = fixture();
  const baseBytes = encode(base),
    headBytes = encode(head);
  const proof = certifyUnaffectedProjectionPackets(baseBytes, headBytes, root, root);
  assert.equal(proof.completePackets, 228);
  assert.equal(proof.unchangedPackets, 152);
  assert.equal(proof.relocatedPackets, 76);
  assert.equal(proof.sharedPackets, 96);
  assert.equal(proof.relocations.length, 76);
  assert.deepEqual(encode(base), baseBytes, "original packets are retained without normalization");
  assert.deepEqual(encode(head), headBytes, "head packets are retained without normalization");
});

for (const [name, mutate] of [
  [
    "diagnostic",
    (p) => {
      p.diagnostics[0].messageText = "changed";
    },
  ],
  [
    "authored mapping",
    (p) => {
      p.mappings[0][2] = 1;
    },
  ],
  [
    "mapping features",
    (p) => {
      p.mappings[0][5] = 0;
    },
  ],
  [
    "semantic link",
    (p) => {
      p.semanticLinks[0].kind = "changed";
    },
  ],
  [
    "directive",
    (p) => {
      p.diagnosticDirectives.directives[0][4] = 1;
    },
  ],
  [
    "extension",
    (p) => {
      p.extension = ".js";
    },
  ],
  [
    "unknown packet field",
    (p) => {
      p.unqualified = true;
    },
  ],
  [
    "unknown generated bytes",
    (p) => {
      p.text += "const unqualified = true;\n";
    },
  ],
  [
    "uncertified Slots export",
    (p) => {
      p.text = p.text.replace("export type { $Slots as Slots };", "export type Slots = unknown;");
    },
  ],
  [
    "uncertified Slots marker",
    (p) => {
      p.text = p.text.replace("Partial<$Slots>", "Partial<unknown>");
    },
  ],
]) {
  test(`Unicode consumer refuses ${name} mutations`, () => {
    const { base, head } = fixture();
    mutate(head[mapperIndex(head)].packet);
    assert.throws(() => replay(base, head));
  });
}

test("Unicode consumer refuses missing, repeated and reidentified packet rows", () => {
  const { base, head } = fixture();
  assert.throws(() => replay(base, head.slice(1)));
  const duplicate = structuredClone(head);
  duplicate[1] = structuredClone(duplicate[0]);
  assert.throws(() => replay(base, duplicate));
  const reidentified = structuredClone(head);
  reidentified[mapperIndex(head)].options = 32;
  assert.throws(() => replay(base, reidentified));
});

for (const file of assets) {
  test(`Unicode source custody covers and refuses changed ${file}`, () => {
    assert.equal(isProjectionSourcePath(file), true);
    const temporary = mkdtempSync(path.join(os.tmpdir(), "unicode-prop-assets-"));
    try {
      for (const role of ["base", "head"]) {
        for (const asset of assets) {
          const destination = path.join(temporary, role, asset);
          mkdirSync(path.dirname(destination), { recursive: true });
          writeFileSync(destination, readFileSync(path.join(root, asset)));
        }
      }
      const changed = path.join(temporary, "head", file);
      writeFileSync(
        changed,
        Buffer.concat([readFileSync(changed), Buffer.from("// unqualified mutation\n")]),
      );
      const { base, head } = fixture();
      assert.throws(() =>
        replay(base, head, path.join(temporary, "base"), path.join(temporary, "head")),
      );
    } finally {
      rmSync(temporary, { recursive: true, force: true });
    }
  });
}

test("Unicode source custody retains the existing Rust and Cargo input authorities", () => {
  for (const file of [
    "tools/support/compat/github/unicode-prop-projection-custody.mjs",
    "tools/support/compat/content-mapper/constructor-projection-parity.ts",
    "crates/vize_canon/src/lib.rs",
    "Cargo.toml",
    "crates/vize_canon/Cargo.toml",
    "Cargo.lock",
    ".cargo/config.toml",
  ])
    assert.equal(isProjectionSourcePath(file), true);
  assert.equal(isProjectionSourcePath("unrelated-history.txt"), false);
});

test("Unicode workflow registers the exact certificate, assets and refusal tests", () => {
  const workflow = readFileSync(
    path.join(root, ".github/workflows/unicode-prop-projection.yml"),
    "utf8",
  );
  for (const file of [
    "tools/support/compat/content-mapper/constructor-projection-parity.ts",
    "tests/tooling/constructor-projection-parity.test.ts",
    "tests/tooling/unicode-prop-projection-custody.test.mjs",
    ...assets,
  ])
    assert.ok(workflow.includes(`      - ${file}\n`), `missing path registration: ${file}`);
  const tests = workflow.indexOf(
    "node --test tests/tooling/constructor-projection-parity.test.ts tests/tooling/unicode-prop-projection-custody.test.mjs",
  );
  const capture = workflow.indexOf(
    "run: node tools/support/compat/github/unicode-prop-projection-custody.mjs",
  );
  assert.ok(
    tests >= 0 && capture > tests,
    "refusal controls must run before both fresh source builds",
  );
});
