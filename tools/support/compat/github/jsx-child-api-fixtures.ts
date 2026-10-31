// Original #6890 production JSX witness; generic SSR and whole-product parity remain unsupported.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const JSX_CHILD_API_CASE_ID = "compiler/jsx/child-values-6890";
export const JSX_CHILD_ADMITTED_REVISION = "f3ba538391bb6b5be2c8506008ccf853e4aecc1a";
const approvedSources = [
  {
    path: "crates/vize_atelier_jsx/src/compile.rs",
    beforeSha256: "0546c2e45f16a9b9c5cd1349905ed2e7bab37bd75f66e2ea07ec4c563e99bdb6",
    afterSha256: "2917dedcc374de54055399e5b5b7207df15101310425766d92dea78ed838bbb6",
  },
  {
    path: "crates/vize_atelier_jsx/src/compile/snapshots/vize_atelier_jsx__compile__tests__typed_slot_runtime_vapor_child_values_6890.snap",
    beforeSha256: null,
    afterSha256: "ebd8f168c75566c4a9a4aa06cf66c66a14e238b8881e46c52e685949c279d96b",
  },
  {
    path: "crates/vize_atelier_jsx/src/compile/snapshots/vize_atelier_jsx__compile__tests__typed_slot_runtime_vapor_child_values_6890_inert_template.snap",
    beforeSha256: null,
    afterSha256: "fba32aa893df329c258497e483a7a7dec7c0ce97664d6f3e276330ed4ce9b2d0",
  },
  {
    path: "crates/vize_atelier_jsx/src/compile/tests.rs",
    beforeSha256: "5c46a1972dcf2f6864d1837ca86ecdb2adf14dddb3baf200dcae1e52ae6b7724",
    afterSha256: "5238eb92bd1f299b8d8b9365fa98d99a7fc71e5e72fd0e2eae956da19b471280",
  },
  {
    path: "crates/vize_atelier_jsx/src/spread_children.rs",
    beforeSha256: "02df4c7de0bfef2265fdc02d03139641cf7d5134eed65b554ba5f136697d4265",
    afterSha256: "e6a9d935b230816319b4c12e6608fca642dc4ab2e870fefc2dabf1f2b263efc8",
  },
  {
    path: "crates/vize_atelier_jsx/src/vapor.rs",
    beforeSha256: "4b72d49e61447e7a292ce62f8bdeebe4d6a382b18e2509c0f5b0af24c66e662f",
    afterSha256: "e7c655a675661c8db2cdb9919bab0e63e396f402d51ec0a7f8efae0fcc62f273",
  },
  {
    path: "crates/vize_atelier_vapor/src/compile/inner.rs",
    beforeSha256: "0530a6749b44b19400042e9a177f0861a6d59965021ea85100cfbef0bff9bbc1",
    afterSha256: "acefc9ab0969b2a437d11beafbd738f6cb61282d6bb9549cad0c1c763cdf7aca",
  },
  {
    path: "crates/vize_atelier_vapor/src/generate.rs",
    beforeSha256: "d708a78dacec3b1669a7c88aff016696747f45d4455cccb679ee375aa382926d",
    afterSha256: "699c15d6b2d8dc98e93b7a868f01832451d6376a34d7171506b497312c399af6",
  },
  {
    path: "crates/vize_atelier_vapor/src/generate/context.rs",
    beforeSha256: "72bb904d84c2588d07366797ff0bab61ce942fa08ca872b6cd5986dbfbd57b4f",
    afterSha256: "639b9aa9d73be804fbc857289e74b986ea3758cf6ac44ab98611a3e7a569d340",
  },
  {
    path: "crates/vize_atelier_vapor/src/generate/effect_order.rs",
    beforeSha256: "5af851208188652db7965b8d9c484e84276022ac5b034712ea29363d0567b973",
    afterSha256: "12b2840f712076e7d601a383cf782d4abe54069fce13d230f606794f8743e5b5",
  },
  {
    path: "crates/vize_atelier_vapor/src/generate/entry.rs",
    beforeSha256: "7a533d1aafbdbbeb0744c3c323077f313a46614f166ff14d27a2cc5b7d43675d",
    afterSha256: "15f4f438ada09b3be220c35ad1ad0403684ccbea7ec12b53349a6fbfb07c36eb",
  },
  {
    path: "crates/vize_atelier_vapor/src/generate/operations.rs",
    beforeSha256: "7a58d2adf2d8da8e1a1aa67ba2d35d0d868e9b2a91e260129addf034d896537f",
    afterSha256: "dc35ad1e8d3020d605bda1e476593ef961e040b281d00ddd927640ff309c7653",
  },
  {
    path: "crates/vize_atelier_vapor/src/generate/operations/jsx_children.rs",
    beforeSha256: null,
    afterSha256: "4fdc9d8e0cb03d003423dc1874f6efb5e5660b02659f40570ca4e520bdddaec2",
  },
  {
    path: "crates/vize_atelier_vapor/src/generate/transformed_root.rs",
    beforeSha256: "063a9eb221c6cb27fd1b88da02cf31633b88cd3e08e4a8588bf270efc9d6ec47",
    afterSha256: "ec09b1bd74519fcf2cf9551027ddf04db305c1c664c66ee0ec47dd7dc9f322a2",
  },
  {
    path: "crates/vize_atelier_vapor/src/ir.rs",
    beforeSha256: "39569db13ae94c194a78e3b0a22aa8625d58b9160c1a66fb3bd1a17e494450c2",
    afterSha256: "34f58208a33f92b942847de013fb83f1cdea4a0b6f0784ddaa0802459a255de6",
  },
  {
    path: "crates/vize_atelier_vapor/src/lower.rs",
    beforeSha256: "52143f26569be26ef447b4b14d2f09835cd6d071a719db32127e82fed51deb2a",
    afterSha256: "2ece62a4fcee7b675c856f8309090925ebf8742704d4d63fba58ff3c0cec81ac",
  },
  {
    path: "crates/vize_atelier_vapor/src/lower/context.rs",
    beforeSha256: "8c3ec53043444858f16b5c8d6ea0160ed3b4b00d47623f37bd93113bd83aa001",
    afterSha256: "51c5788cb831a76814383f11988b2f31be7b28af5673daa731b4625f7c691f83",
  },
  {
    path: "crates/vize_atelier_vapor/src/lower/text.rs",
    beforeSha256: "c9c39fe467b4cf6582a40e7910098b2e1b86435f9cf7d93ddc6670f1329030b5",
    afterSha256: "ee105ba911049e414e25d62f2f93f7b5bcdf6a1396e75d72ba23c7c4b34a71d1",
  },
] as const;
const approvedInputs = [
  {
    path: "tests/_fixtures/differential/compiler/jsx-child-values-6890/original.json",
    sha256: "810595733fc8904df998e7d2a1ac1d70319c0be22046d6cb25845b6c46697a37",
  },
  {
    path: "tests/_fixtures/differential/jsx/child-values-6890-original.json",
    sha256: "810595733fc8904df998e7d2a1ac1d70319c0be22046d6cb25845b6c46697a37",
  },
  {
    path: "tests/_fixtures/differential/jsx/child-values-6890-reference-conflict.json",
    sha256: "41e92a4b0f2c7d3e8a5deedeb88941146f04b10fa4ee9d066478e78f79e3df6c",
  },
  {
    path: "tests/_fixtures/differential/compiler/jsx-child-values-6890/browser-before-f7c306d.json",
    sha256: "47106a01b905c1036ad8c0b79c80928b967b799b0f03a1f91dbd5439a41b0a71",
  },
  {
    path: "tests/_fixtures/differential/compiler/jsx-child-values-6890/rejected-spread.jsx.txt",
    sha256: "ae82ba0cb7d56386f68ff05fdcb54e407c6f0708c73b51fa3c9540b65876bfbb",
  },
] as const;
const approvedWitnesses = [
  {
    path: "crates/vize_atelier_jsx/tests/dynamic_child_values.rs",
    sha256: "f8202c0b48f78961f07b54c6967ff0adf219c5cfbe4d4907b454ee5544a53a0e",
  },
  {
    path: "tests/tooling/jsx-vapor-oracle/child-values.ts",
    sha256: "4061aaa141e52b17c170fe74bb4cc6532f9f1c95e5363d38691392f0f2b42bf6",
  },
  {
    path: ".github/workflows/jsx-authored-native-modules.yml",
    sha256: "3fc99398d3ede7e986be57a4ad1e723610afe330609a1a0d4ec79445fc7cef39",
  },
  {
    path: "tests/tooling/jsx-vapor-oracle/child-browser.ts",
    sha256: "055f401b4e6c23a46945e8937e32eda947892d321752f3bb2d9576fe1fa4069d",
  },
  {
    path: "crates/vize_atelier_jsx/tests/spread_child.rs",
    sha256: "b0f58eb035941487bbc387628139f87613226c42d577074aec868d1f2c2d07a7",
  },
] as const;
const qualification = {
  originalPhases: 10,
  qualifiedVaporPhases: 9,
  referenceConflictPhases: 1,
  wholeCorpusQualified: false,
  issue6890Complete: false,
};
const scope =
  "source-built JSX Vapor API/runtime witness; nine common phases; one retained reference conflict; no native VDOM, generic SSR, or whole-product parity credit";
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function matches(read: ReadFile, side: "base" | "head", path: string, expected: string | null) {
  const bytes = read(side, path);
  return expected === null ? bytes === null : bytes !== null && digest(bytes) === expected;
}

export function newJsxChildApiCases(
  previous: unknown,
  current: unknown,
  read: ReadFile,
): { ids: string[]; sources: Set<string> } {
  const result = { ids: [] as string[], sources: new Set<string>() };
  const actualPair = approvedSources.every(
    (source) =>
      matches(read, "base", source.path, source.beforeSha256) &&
      matches(read, "head", source.path, source.afterSha256),
  );
  if (current === undefined) {
    assert(!actualPair, "the actual JSX child producer requires its original input registration");
    return result;
  }
  assert(Array.isArray(current), "jsxChildApiCases must be an array");
  assert(previous === undefined || Array.isArray(previous));
  const oldIds = new Set((previous ?? []).map((row: { id?: unknown }) => row?.id));
  const seen = new Set<string>();
  for (const value of current) {
    assert(value !== null && typeof value === "object" && !Array.isArray(value));
    assert.deepEqual(
      Object.keys(value).sort(),
      [
        "id",
        "state",
        "adapter",
        "targets",
        "scope",
        "qualification",
        "inputs",
        "witnesses",
        "sources",
      ].sort(),
      "closed JSX child registration schema",
    );
    const row = value as Record<string, unknown>;
    assert(typeof row.id === "string" && !seen.has(row.id), "missing/duplicate JSX child ID");
    seen.add(row.id);
    if (oldIds.has(row.id)) continue;
    assert.equal(row.id, JSX_CHILD_API_CASE_ID, "unregistered future JSX family");
    assert.equal(row.state, "active");
    assert.equal(row.adapter, "jsx-vapor-child-runtime-source-v1");
    assert.deepEqual(row.targets, ["vapor-csr"]);
    assert.equal(row.scope, scope);
    assert.deepEqual(
      row.qualification,
      qualification,
      "retained ten phases and open issue boundary",
    );
    assert(actualPair, "stale or different actual JSX producer source pair");
    assert.deepEqual(row.sources, approvedSources, "finite byte-pinned producer closure");
    assert.deepEqual(
      row.inputs,
      approvedInputs,
      "original input and reference conflict are immutable",
    );
    assert.deepEqual(row.witnesses, approvedWitnesses, "real executable consumers are immutable");
    for (const pin of [...approvedInputs, ...approvedWitnesses]) {
      assert(
        matches(read, "head", pin.path, pin.sha256),
        `missing/stale JSX input or witness: ${pin.path}`,
      );
    }
    const original = read("head", approvedInputs[0].path)!;
    assert.deepEqual(
      original,
      read("head", approvedInputs[1].path),
      "original corpus bytes retained",
    );
    const canonicalOriginal = Buffer.from(JSON.stringify(JSON.parse(original.toString("utf8"))));
    assert.equal(
      digest(canonicalOriginal),
      "9c7f9c6593fef9d8962be316a99e4066dd4f6e75d4936f2a028538c504105712",
    );
    const producer = read("head", approvedWitnesses[0].path)!.toString("utf8");
    assert(
      producer.includes("fn original_child_values_match_vapor_with_an_explicit_vnode_frontier("),
    );
    assert(producer.includes("include_str!("));
    assert(
      producer.includes(
        "../../../tests/_fixtures/differential/compiler/jsx-child-values-6890/original.json",
      ),
    );
    assert(producer.includes("tests/tooling/jsx-vapor-oracle/child-values.ts"));
    const workflow = read("head", approvedWitnesses[2].path)!.toString("utf8");
    assert(workflow.includes("vue_jsx_vapor_compat_oracle dynamic_child_values"));
    assert(workflow.includes("ordered_spread_props spread_child"));
    const refusal = read("head", approvedWitnesses[4].path)!.toString("utf8");
    assert(
      refusal.includes("fn vapor_and_ssr_report_the_spread_instead_of_emitting_client_blocks("),
    );
    assert(refusal.includes("/jsx-child-values-6890/rejected-spread.jsx.txt"));
    result.ids.push(row.id);
    for (const source of approvedSources) result.sources.add(source.path);
  }
  if (actualPair)
    assert(result.ids.includes(JSX_CHILD_API_CASE_ID), "missing fresh JSX child witness");
  return result;
}
