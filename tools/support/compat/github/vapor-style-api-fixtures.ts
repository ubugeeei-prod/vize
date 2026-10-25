// Closed original-AST Vapor CSR source witness; no generic SSR adapter credit.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const VAPOR_STYLE_API_CASE_ID = "compiler/vapor/literal-object-styles-10749";
const approvedSources = [
  {
    path: "crates/vize_atelier_core/src/lane/element.rs",
    beforeSha256: "614da41dc6908c8a8dcb989daaa930d1daca88dd05fce48cdf3156b319df514e",
    afterSha256: "d98ca26debc28aceedefc8263ab94d7175ff87ae8d20b7c5c2290e616f87aef9",
  },
  {
    path: "crates/vize_atelier_core/src/lane/element/vapor_static_style.rs",
    beforeSha256: null,
    afterSha256: "646962e6669edff43d3f30e2f290b7644bed190472a1898bf74f9e05e452a757",
  },
  {
    path: "crates/vize_atelier_core/src/lib.rs",
    beforeSha256: "8a295308f7f422e6c7c7b95e71543faaa0c1acd31d0cc2114710cafd162c9d33",
    afterSha256: "807fb9af9bc8b02ccd5cbeca049367501f293c957716cc256dccaa7a3720a92c",
  },
  {
    path: "crates/vize_atelier_jsx/src/compile.rs",
    beforeSha256: "31bade7b621bfe6c5ed5b5553d4d3b7a3c6ce42d7e63f5e33349177784b72651",
    afterSha256: "a8da2febd1a236316e3d206a12c81d0e318b51053ecde31d3f89693ea673bb03",
  },
  {
    path: "crates/vize_atelier_jsx/src/lower.rs",
    beforeSha256: "9af1f9f9dedd946348ca142eeac46b2e4a4ccb1ea24a35d6dc2e88ae171c79c3",
    afterSha256: "b9720550ae1dbdb8705436d44e4579f4882ac453616da7e09caa3f5892dbb087",
  },
  {
    path: "crates/vize_atelier_jsx/src/lower/attr.rs",
    beforeSha256: "e36da4de9d3339d2e698167bc3e728b45180c5eba732f07693bc3f9cfb7aafa2",
    afterSha256: "446edfb793140220681335fdf8b73574f1bc95746518594d48738cf9ceed54ec",
  },
  {
    path: "crates/vize_atelier_jsx/src/lower/attr/static_style.rs",
    beforeSha256: null,
    afterSha256: "9468e859a7f19aa4fa6cc8b2f3fef1b5231b43d3e591571a931fb625dfc03c53",
  },
  {
    path: "crates/vize_atelier_jsx/src/vapor.rs",
    beforeSha256: "c9777eb8e5fadff520ac988af24f3f7d2982c64701a133b68896e2ecb4436371",
    afterSha256: "44307ba62db9b8640ddbf251822a95fb9e9170acc6dcd97e40c15aaf08902736",
  },
  {
    path: "crates/vize_atelier_vapor/src/l3/native/emit.rs",
    beforeSha256: "605b868befbffa307e0b639470c386a1558ccf7ee431f0454391ce7fc6683262",
    afterSha256: "7a298ea85bb68cb0055258e37952170b4581d2a221c094c18bf011002164c84d",
  },
  {
    path: "crates/vize_atelier_vapor/src/l3/native/emit/element.rs",
    beforeSha256: "57356e6432c70c8bc8c4e599c3ff03b4477440fb82f73b86193b6e2386f55686",
    afterSha256: "cd18eb9b12c3952d112b77e2e9c130f6024823ae354086735b95b2bb997d8c91",
  },
  {
    path: "crates/vize_atelier_vapor/src/l3/native/emit/static_style.rs",
    beforeSha256: null,
    afterSha256: "b8069a9df5326dad0c13027679618151d73f68ac48567a4ac97707355e351b07",
  },
  {
    path: "crates/vize_relief/src/inline_style.rs",
    beforeSha256: "17db6757ce83bf81a80ed5d6bee44f2cbbd0e130a64d71c16b2f01da25b33bf7",
    afterSha256: "577c49701667a5d010966e34cfcb7bba61cd489653a025256d702769fc326efb",
  },
  {
    path: "crates/vize_relief/src/inline_style/literal.rs",
    beforeSha256: null,
    afterSha256: "28b2c954429ca3fd218ae5458fac2a98715822fdf38f55a94eaf543de80d8fb0",
  },
  {
    path: "crates/vize_relief/src/lib.rs",
    beforeSha256: "9e8927ac250185e6ec3e992fa102e09dc6aaab5e691c586dc0fcb8d73b172610",
    afterSha256: "67277b24381b9b8887b89db35ad20c3ff6bc817f1c1fa44a4d505a2d26c7bf94",
  },
] as const;
const approvedInputs = [
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/App.vue.txt",
    sha256: "555fd69bae5d841628e76aec534fa3d98f3033ca2014fa4ab749862970413a3f",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/admission-source-pairs.json",
    sha256: "f4b89df97e69df1a6f08bfa81a0bfbab56ad272a8686adbbef425864dae7bac7",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/attribute-case-inputs.json",
    sha256: "d8510c242246ef26e1639ce5bd3d69b562837f78437fbcec01bed4b119b3f9ad",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/authority.json",
    sha256: "e113463a40ba2d54d8c8a82e81f9b5a42e5f88900e1850a1c4ccd791ab0aa01b",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/inputs.json",
    sha256: "f5573ccf312011736b6869b4ae0b3005df75c3f2f7791e195512bdd211cca11b",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/observed-base.json",
    sha256: "f79947fcb0a25776867e4d2a91566474eb70684c7ecf87683dcd86406bb4601d",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/observed-comparison.json",
    sha256: "19463ed6fb9109c7dd8d47beec199d9d08041d326785e0a673816da2010d4597",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/observed-head.json",
    sha256: "18244764c91eb3d4ff9451bb6811a9e7840375e9d5af01cd842fc82834c5e63e",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/observed-measurement.json",
    sha256: "6294b18d69e400369ac8ca3bdf6373c1294fdcddcca300dd82676a152150f764",
  },
  {
    path: "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/review-inputs.json",
    sha256: "70d1f7b95a94080628a7134d7d44722f11453f6c7e8f25be52b4e6fb08699b66",
  },
] as const;
const approvedWitnesses = [
  {
    path: "crates/vize_atelier_jsx/tests/literal_style_source.rs",
    symbol: "complete_literal_style_source_observations",
    sha256: "f65aff47a6b28867a63c82ffc4e7dabf546e6438b63bb753ee89f36b2f843ce3",
  },
  {
    path: "crates/vize_atelier_jsx/tests/literal_object_styles.rs",
    symbol: "original_literal_ast_has_canonical_object_and_css_semantics",
    sha256: "7cde2ce71c1c61eeb4e4cd5d8062be0e455aa4ab79108e6f76556c1ec573b1d3",
  },
  {
    path: "crates/vize_atelier_jsx/tests/literal_style_maps.rs",
    symbol: "folded_css_has_real_complete_utf16_origins_and_rejects_mutations",
    sha256: "badda2c866e4bf68b7ee8e11f09a1c6bbd2dae805c2f25157f39263e96fdf693",
  },
  {
    path: "crates/vize_atelier_jsx/tests/literal_style_sfc.rs",
    symbol: "original_sfc_literal_style_reaches_the_shared_static_template",
    sha256: "03b8f2d1250db9129485a73d2e63cdc4944e50cfa75da84c1af05718f4c24c90",
  },
] as const;
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function object(value: unknown, keys: readonly string[]): Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  assert.deepEqual(Object.keys(value).sort(), [...keys].sort(), "closed Vapor style schema");
  return value as Record<string, unknown>;
}
function matches(read: ReadFile, side: "base" | "head", path: string, expected: string | null) {
  const bytes = read(side, path);
  return expected === null ? bytes === null : bytes !== null && digest(bytes) === expected;
}

export function newVaporStyleApiCases(
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
    assert(!actualPair, "actual Vapor style source pair requires its real input registration");
    return result;
  }
  assert(Array.isArray(current), "vaporStyleApiCases must be an array");
  assert(previous === undefined || Array.isArray(previous));
  const oldIds = new Set((previous ?? []).map((row: { id?: unknown }) => row?.id));
  const seen = new Set<string>();
  for (const value of current) {
    const row = object(value, [
      "id",
      "state",
      "adapter",
      "targets",
      "scope",
      "inputs",
      "witnesses",
      "sources",
    ]);
    assert(typeof row.id === "string" && !seen.has(row.id), "missing/duplicate Vapor style ID");
    seen.add(row.id);
    if (oldIds.has(row.id)) continue;
    assert.equal(row.id, VAPOR_STYLE_API_CASE_ID, "unregistered future Vapor style family");
    assert.equal(row.state, "active");
    assert.equal(row.adapter, "shared-vapor-literal-style-source-v1");
    assert.deepEqual(row.targets, ["vapor-csr"]);
    assert.equal(
      row.scope,
      "source-only; generic SSR CLI unsupported; no whole-product native or browser parity credit",
    );
    assert(actualPair, "stale or different actual producer source pair");
    assert.deepEqual(row.sources, approvedSources, "source closure is finite and byte-pinned");
    assert.deepEqual(
      row.inputs,
      approvedInputs,
      "actual fixed inputs and historical packets are immutable",
    );
    assert.deepEqual(
      row.witnesses,
      approvedWitnesses,
      "real complete executable witnesses are immutable",
    );
    for (const pin of [...approvedInputs, ...approvedWitnesses]) {
      assert(
        matches(read, "head", pin.path, pin.sha256),
        `missing/stale actual input or witness: ${pin.path}`,
      );
    }
    for (const witness of approvedWitnesses) {
      const source = read("head", witness.path)!.toString("utf8");
      assert(source.includes(`fn ${witness.symbol}(`));
    }
    const sourceObserver = read("head", approvedWitnesses[0].path)!.toString("utf8");
    assert(sourceObserver.includes("include_str!("));
    assert(sourceObserver.includes("/vapor-literal-object-styles-10749/inputs.json"));
    result.ids.push(row.id);
    for (const source of approvedSources) result.sources.add(source.path);
  }
  if (actualPair)
    assert(result.ids.includes(VAPOR_STYLE_API_CASE_ID), "missing fresh real Vapor style witness");
  return result;
}
