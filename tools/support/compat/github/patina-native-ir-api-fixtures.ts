// One closed Rust API correction, with actual required Rust consumers. This
// registers execution; only the Rust job results provide execution evidence.
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;

export const PATINA_NATIVE_IR_API_ID = "linter/rust-api/native-ir-accessors-13674";
export const patinaNativeIrApiRoot =
  "tests/_fixtures/differential/linter/native-ir-accessor-compatibility-13674";

export const patinaNativeIrApiConsumer = "crates/vize_patina/tests/native_ir_accessor_compat.rs";

export const patinaNativeIrApiRegistration = {
  id: PATINA_NATIVE_IR_API_ID,
  state: "active",
  adapter: "patina-native-ir-accessor-compatibility-v1",
  targets: ["rust-api"],
  inputs: {
    root: "native-ir-accessor-compatibility-13674",
    files: [
      {
        path: `${patinaNativeIrApiRoot}/cases.json`,
        role: "getter-contract",
        sha256: "eddc4d895ebe62af5e27cc516c24a9218929ca7f54e60da90e0db5046fbd1802",
      },
      {
        path: patinaNativeIrApiConsumer,
        role: "required-rust-consumer",
        sha256: "6bb113c95205e96ca08da77925c27cee3251bbb832ef5b2b34cdca20bc4947b5",
      },
    ],
  },
  sources: [
    {
      path: "crates/vize_patina/Cargo.toml",
      before: "cc18cbfda5a62c25c26becc256e84cc0cf38a513458bd5ec25494b77e72eeb0a",
      after: "cc18cbfda5a62c25c26becc256e84cc0cf38a513458bd5ec25494b77e72eeb0a",
      beforeBlob: "82f140b193f83cfef4b52266a1c7353df9c67b60",
      afterBlob: "82f140b193f83cfef4b52266a1c7353df9c67b60",
    },
    {
      path: "crates/vize_patina/src/native/canonical.rs",
      before: "2265d2e4d8b5256c79f0af255722547e6d041625c9d9579e0e835be7278ed78d",
      after: "b83dcf8515bc6ce496b36bf27fcef7ff05b747b855495f226d092bcea3d63aca",
      beforeBlob: "3eea9726453a5722846327831deac71131814771",
      afterBlob: "67edc985d55e071fd9cd3e18dbabd52b68d25487",
    },
    {
      path: "crates/vize_patina/src/native/sfc/owner.rs",
      before: "d4a6114c7910e3e6dc5835201dc072df458a7e2a57e8fe12d3360c5e11fb4348",
      after: "26e6de88fc0fc3775cdc52f983ddcef71dfbfd9810616397d27d1afaf29ff2ba",
      beforeBlob: "b3d42282d9d409ad3c4fb59e1d63fa8fedf74f95",
      afterBlob: "a0106ca31e39772844cdbc04ca8058691cc2e086",
    },
    {
      path: "crates/vize_patina/src/native/sfc/template/owner.rs",
      before: "7cdc02f191d5efb61439a2f80d66adb7e1f0739ba2583b0db90c9dc4dd4a67d9",
      after: "0b9e221b9590f6227b69dc87805999a1bdf414934c30e8ac0a2ce1330a1c1293",
      beforeBlob: "1b4f192479342a1ac66e7d83925f8af73f364112",
      afterBlob: "b0c07093d2b234fc6d2499daf830041adf80afc1",
    },
    {
      path: "crates/vize_patina/src/native/sfc/tests/ownership.rs",
      before: "a01a66817e3c8af1ff676ae80e25882b92fd1a89de7287b14ef3698c46136984",
      after: "f38174d54e2dbe2566c689ad7c35034497dc2d1c89fe445d0781e34f4e125a5e",
      beforeBlob: "bb14a9c8ccaddb522563d1c746d60386fc4ab9dc",
      afterBlob: "e7688ed8ada6f4eff747f5715184121fed038f67",
    },
    {
      path: "crates/vize_patina/tests/main.rs",
      before: "eb08136b9d56102e8af28fbd8718f8751288c24d55c929cb01dfcb10659a713a",
      after: "5eaddd7f8cf594e47df49b289410d2158d4a9316b2536cc64ccb9d289fae4331",
      beforeBlob: "7550539dca2779eb5e1805b8420ca68db1e2ae51",
      afterBlob: "9d7872d68e22b69835d4e713066df9608be5da86",
    },
    {
      path: "crates/vize_patina/tests/native_sfc_template_lint/configuration.rs",
      before: "dc27be82ce4649b5f38eed23ec656c2a1d02f8cfeeb5b410edf44324be5da18e",
      after: "9bfb72fd6a6652a0b0e5ff5f92b89f2c9ee714212ded6beeb611ccdca938e80e",
      beforeBlob: "b9b8e1f87504944e832a7987c8fbac3134517491",
      afterBlob: "0f421a5c89b35391ca5a4743804b38502b0e02c0",
    },
  ],
  tests: [
    {
      target: "integration",
      selector:
        "native_ir_accessor_compat::old_owner_getters_remain_public_with_the_original_borrow_signatures",
      source: patinaNativeIrApiConsumer,
    },
    {
      target: "integration",
      selector:
        "native_ir_accessor_compat::old_canonical_getter_retains_the_owner_borrow_after_the_lint_wrapper_is_dropped",
      source: patinaNativeIrApiConsumer,
    },
    {
      target: "lib",
      selector:
        "native::sfc::tests::ownership::normally_owned_program_and_file_origin_survive_owner_move_before_a_short_reborrow",
      source: "crates/vize_patina/src/native/sfc/tests/ownership.rs",
    },
    {
      target: "integration",
      selector:
        "native_sfc_template_lint::configuration::literal_text_requires_every_actual_callback_in_either_registry_order",
      source: "crates/vize_patina/tests/native_sfc_template_lint/configuration.rs",
    },
  ],
  evidence: {
    predecessor: "source-authenticated-only",
    current: "registered-required-rust-consumer",
    languageParity: "not-claimed",
  },
};

const expectedData = {
  schema: "vize.patina.native-ir-accessor-compatibility",
  version: 1,
  getters: [
    { owner: "NativeCanonicalLint", legacy: "file", canonical: "l2ir" },
    { owner: "NativeSfcLintOwner", legacy: "descriptor", canonical: "l1ir" },
    { owner: "NativeSfcLintOwner", legacy: "file", canonical: "l2ir" },
    { owner: "NativeSfcTemplateOwner", legacy: "descriptor", canonical: "l1ir" },
  ],
};

function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`invalid Patina Rust API case: ${message}`);
}

function object(value: unknown, keys: readonly string[]): Record<string, unknown> {
  require(value && typeof value === "object" && !Array.isArray(value), "object required");
  require(JSON.stringify(Object.keys(value).sort()) ===
    JSON.stringify([...keys].sort()), "closed schema fields differ");
  return value as Record<string, unknown>;
}

function equal(actual: unknown, expected: unknown, message: string): void {
  require(JSON.stringify(actual) === JSON.stringify(expected), message);
}

function pinned(
  readFile: ReadFile,
  side: "base" | "head",
  path: string,
  expected: string,
  expectedBlob?: string,
) {
  const bytes = readFile(side, path);
  require(bytes !== null, `missing ${side} input: ${path}`);
  require(createHash("sha256").update(bytes).digest("hex") ===
    expected, `${side} hash mismatch: ${path}`);
  if (expectedBlob !== undefined) {
    require(createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex") ===
      expectedBlob, `${side} blob identity mismatch: ${path}`);
  }
  return bytes;
}

export function newPatinaNativeIrApiCases(
  previous: unknown,
  current: unknown,
  readFile: ReadFile,
): { ids: string[]; sources: Set<string> } {
  const result = { ids: [] as string[], sources: new Set<string>() };
  if (current === undefined && previous === undefined) return result;
  require(Array.isArray(current), "rustApiCases must be an array");
  require(previous === undefined || Array.isArray(previous), "prior cases must be an array");
  const old = previous ?? [];
  require(current.length <= 1 && old.length <= 1, "only the registered case is supported");
  require(current.length >= old.length, "cannot drop the historical registration");
  for (const value of current) {
    const row = object(value, Object.keys(patinaNativeIrApiRegistration));
    require(row.id === PATINA_NATIVE_IR_API_ID, "unregistered ID");
    equal(row.state, "active", "inactive case");
    equal(row.adapter, patinaNativeIrApiRegistration.adapter, "unregistered adapter");
    equal(row.targets, patinaNativeIrApiRegistration.targets, "unregistered targets");
    equal(row.inputs, patinaNativeIrApiRegistration.inputs, "input identities differ");
    equal(row.sources, patinaNativeIrApiRegistration.sources, "source closure differs");
    equal(row.tests, patinaNativeIrApiRegistration.tests, "selector closure differs");
    equal(row.evidence, patinaNativeIrApiRegistration.evidence, "evidence scope differs");
    for (const input of patinaNativeIrApiRegistration.inputs.files) {
      pinned(readFile, "head", input.path, input.sha256);
    }
    if (old.length) {
      equal(value, old[0], "historical registration changed");
      continue;
    }

    const dataPath = `${patinaNativeIrApiRoot}/cases.json`;
    const rawData = readFile("head", dataPath);
    require(rawData !== null, "missing getter data");
    equal(JSON.parse(rawData.toString("utf8")), expectedData, "getter contract differs");
    for (const source of patinaNativeIrApiRegistration.sources) {
      pinned(readFile, "base", source.path, source.before, source.beforeBlob);
      pinned(readFile, "head", source.path, source.after, source.afterBlob);
    }

    // This wiring is registration evidence, never a substitute for executing
    // the fixture's real consumer in the required lib/integration Rust tests.
    const read = (path: string) => readFile("head", path)?.toString("utf8") ?? "";
    require(read("crates/vize_patina/Cargo.toml").includes(
      '[[test]]\nname = "integration"\npath = "tests/main.rs"',
    ), "Rust integration target is not registered");
    require(read("crates/vize_patina/tests/main.rs").includes(
      "mod native_ir_accessor_compat;",
    ), "Rust consumer is not registered in the integration target");
    require(read(patinaNativeIrApiConsumer).includes(
      "/../../tests/_fixtures/differential/linter/native-ir-accessor-compatibility-13674/cases.json",
    ), "Rust consumer does not read getter data");
    require(read("crates/vize_patina/src/native/sfc/tests/ownership.rs").includes(
      "/../../tests/_fixtures/differential/linter/native-ir-accessor-compatibility-13674/cases.json",
    ), "moved-owner Rust test does not read getter data");
    require(read("crates/vize_patina/tests/native_sfc_template_lint/configuration.rs").includes(
      "crate::native_ir_accessor_compat::assert_template_alias(owner)",
    ), "real template callback does not invoke the fixture consumer");
    result.ids.push(PATINA_NATIVE_IR_API_ID);
    for (const source of patinaNativeIrApiRegistration.sources) result.sources.add(source.path);
  }
  return result;
}
