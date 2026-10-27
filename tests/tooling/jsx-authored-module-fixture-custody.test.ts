import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  fields,
  hash,
  snapshotBody,
  verifyAdversarial,
  verifyStaticSuccessor,
  type PacketFacet,
  type ModulePacket,
  type SuccessorRow,
  type RefusalManifest,
  type RuntimeBaseline,
  type AdversarialRow,
  type AdversarialPacket,
  type AdversarialAuthority,
  type StaticAuthority,
  type LegacyRoles,
  type RoleManifest,
} from "./support/jsx-authored-custody.ts";

function replaceFacet<K extends PacketFacet>(
  target: ModulePacket,
  source: ModulePacket,
  key: K,
): void {
  target[key] = source[key];
}

const nested = new URL(
  "../_fixtures/differential/typecheck/jsx-nested-native-expressions-8441/",
  import.meta.url,
);
const falsy = new URL("../_fixtures/differential/jsx/", import.meta.url);
const adversarial = new URL("authored-native-modules-10700/adversarial/", falsy);
const snapshots = new URL("../../crates/vize_atelier_jsx/tests/snapshots/", import.meta.url);
const read = (base: URL, path: string) => readFileSync(new URL(path, base));
const parse = <T>(base: URL, path: string): T => JSON.parse(read(base, path).toString("utf8")) as T;
const manifest = parse<RefusalManifest>(nested, "authored-module-refusal-10701-successor.json");
const packetFields = ["components", "diagnostics", "module", "moduleMap"];
const componentFields = ["code", "map", "mode", "name", "preamble", "setup", "style"];
const provenance = {
  runId: 40221673116,
  artifactId: 12609354973,
  artifactName: "jsx-authored-native-40221673116-1",
  headSha: "664c7dc3846c068a756b7562e11eab35a9497d9d",
  checkoutSha: "664c7dc3846c068a756b7562e11eab35a9497d9d",
  checkoutTree: "3e1d3d673c142beb6653ad8ef47109f81850e99d",
};
const specs = [
  {
    name: "AfsStepperDialog.stories",
    inputSha256: "8b46599f16d4ac31ed91677a729c9058087ebf2978fd5e62490d512db9e04b8f",
    historicalPacket:
      "AfsStepperDialog.stories.vapor.forwarded-slot-boundaries-9455.runtime.json.txt",
    historicalSha256: "24415129bd50e1f4e1d047dd95e09659b47951576d9ce90c8a8f02eca2305ff3",
    successorSha256: "b0d3349e49fd3fe220ccf12febbc5627a878e78bde8159e93f0377c09a3d321e",
    allowedChangedFacets: ["diagnostics"],
    message:
      "Vapor/SSR component props require an identifier with an explicit nonempty inline object type; destructured, inferred and aliased props need a reactive binding projection",
    start: 164,
    end: 188,
    componentCount: 5,
  },
  {
    name: "native-nested-controls",
    inputSha256: "b107ba38eab500483adb74742186eae4ea4791ee21f3ddbcadf0f3edafa22952",
    historicalPacket: "native-nested-controls.vapor.runtime.json.txt",
    historicalSha256: "1cc74e8645519479e2b80cabeb9fead93d1ef27f0b8b5068c9445049d11748f8",
    successorSha256: "68160b419e195ac9bfdcd31aa0348c67b6dbcce87732ab869db8eb67f2ef5616",
    allowedChangedFacets: ["diagnostics", "module"],
    message:
      "Vapor/SSR authored modules require a directly returned JSX root in an owned component arrow; function declarations, callbacks and other expression contexts are not represented yet",
    start: 32,
    end: 266,
    componentCount: 1,
  },
];

function verifySuccessor(
  spec: (typeof specs)[number],
  row: SuccessorRow,
  historical: ModulePacket,
  successor: ModulePacket,
  source: Buffer,
): void {
  fields(row, [
    "name",
    "input",
    "inputSha256",
    "mode",
    "historicalPacket",
    "historicalSha256",
    "successorPacket",
    "successorSha256",
    "allowedChangedFacets",
    "refusal",
    "qualification",
  ]);
  assert.equal(row.name, spec.name);
  assert.equal(row.input, `${spec.name}.tsx.txt`);
  assert.equal(row.inputSha256, spec.inputSha256);
  assert.equal(row.mode, "vapor");
  assert.equal(row.historicalPacket, spec.historicalPacket);
  assert.equal(row.historicalSha256, spec.historicalSha256);
  assert.equal(
    row.successorPacket,
    `${spec.name}.vapor.authored-module-refusal-10701.runtime.json.txt`,
  );
  assert.equal(row.successorSha256, spec.successorSha256);
  assert.deepEqual(row.allowedChangedFacets, spec.allowedChangedFacets);
  fields(row.refusal, ["message", "start", "end", "sourceSliceSha256"]);
  assert.deepEqual(row.refusal, {
    message: spec.message,
    start: spec.start,
    end: spec.end,
    sourceSliceSha256: hash(source.subarray(spec.start, spec.end)),
  });
  assert.deepEqual(row.qualification, {
    module: "refused",
    components: "inspectable-producer-only",
    runtime: false,
    nativeHandled: 0,
  });
  for (const packet of [historical, successor]) {
    fields(packet, packetFields);
    assert.equal(packet.components.length, spec.componentCount);
    for (const component of packet.components) fields(component, componentFields);
  }
  assert.equal(successor.module, "");
  assert.equal(successor.moduleMap, null);
  assert.equal(
    successor.diagnostics,
    `[JsxDiagnostic { severity: Error, message: "${spec.message}", start: ${spec.start}, end: ${spec.end} }]`,
  );
  const restored = structuredClone(successor);
  for (const facet of row.allowedChangedFacets) {
    assert.ok(packetFields.includes(facet));
    replaceFacet(restored, historical, facet as PacketFacet);
  }
  assert.deepEqual(restored, historical, "every unlisted packet facet stays exact");
  assert.deepEqual(
    packetFields.filter(
      (facet) => !assertSame(historical[facet as PacketFacet], successor[facet as PacketFacet]),
    ),
    spec.allowedChangedFacets,
  );
}

function assertSame(left: unknown, right: unknown): boolean {
  try {
    assert.deepEqual(left, right);
    return true;
  } catch (error) {
    if (error instanceof assert.AssertionError) return false;
    throw error;
  }
}

void test("native refusal successors preserve literal capture and every historical packet", () => {
  fields(manifest, ["schemaVersion", "pullRequest", "historicalBaseline", "capture", "cases"]);
  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.pullRequest, "https://github.com/ubugeeei-prod/vize/pull/10701");
  assert.deepEqual(manifest.capture, {
    ...provenance,
    packet: "nested-packets.json",
    sha256: "8dc2c5c5a37b12b530ac3eb612d48192d3a4ecc0830fa079fa385f2aabbe0f15",
    entry: "compile_jsx -> module_code/source_map/components",
    serialization: "verbatim current object from raw capture, followed by LF",
  });
  const baseline = parse<RuntimeBaseline>(nested, "runtime-baseline.json");
  assert.deepEqual(manifest.historicalBaseline, {
    packet: "runtime-baseline.json",
    sha256: "526a17e43f403f136849d6560ca25ca330d768d550d3cb4958181c24a94ec9b5",
    producer: {
      sourceHead: baseline.sourceHead,
      productionHead: baseline.productionHead,
      nativeProbeSha256: baseline.nativeProbeSha256,
    },
  });
  assert.equal(hash(read(nested, "runtime-baseline.json")), manifest.historicalBaseline.sha256);
  for (const row of baseline.cases) {
    assert.equal(hash(read(nested, row.input)), row.inputSha256);
    assert.equal(hash(read(nested, row.output)), row.outputSha256);
  }
  assert.equal(manifest.cases.length, 2, "only the two reviewed Vapor refusals have successors");
  for (const [index, row] of manifest.cases.entries()) {
    const source = read(nested, row.input);
    const historical = read(nested, row.historicalPacket);
    const successor = read(nested, row.successorPacket);
    assert.equal(hash(source), row.inputSha256);
    assert.equal(hash(historical), row.historicalSha256);
    assert.equal(hash(successor), row.successorSha256);
    verifySuccessor(
      specs[index],
      row,
      JSON.parse(historical.toString("utf8")),
      JSON.parse(successor.toString("utf8")),
      source,
    );
  }
});

void test("adversarial successors retain whole structural observations and frozen originals", () => {
  const stability = new URL("authored-native-modules-10700/", falsy);
  const stabilityAuthority = parse<{
    runtime: boolean;
    rawCaptureSha256: string;
    goldenSha256: string;
  }>(stability, "component-stability-custody.json");
  assert.equal(stabilityAuthority.runtime, false);
  assert.equal(
    stabilityAuthority.rawCaptureSha256,
    "bf8705e52287eeb7a1d6c64908a12f60de997b5825bdbfb30e97cfa6d59dc3ed",
  );
  assert.equal(hash(read(stability, "component-stability.json")), stabilityAuthority.goldenSha256);
  assert.equal(
    hash(read(adversarial, "custody.json")),
    "e5f10f8def638e6b95557ec4dbf55c423e7db8eb0b68e9f697c43ffc78a699e0",
  );
  const authority = parse<AdversarialAuthority>(adversarial, "custody.json");
  assert.equal(authority.schema, 1);
  assert.equal(authority.issue, 10700);
  assert.equal(authority.pullRequest, 10701);
  assert.deepEqual(authority.capture, {
    runId: 40224832845,
    artifactId: 12610971331,
    head: "a4e223413856dd05826474a70e0acb248732003a",
    entry:
      "unchanged adversarial source -> compile_jsx/compile_to_vapor; complete summary/module captured before assertion",
    qualification:
      "structural source observation only; undefined capture fixture is not runtime evidence",
  });
  assert.equal(authority.cases.length, 3);
  for (const row of authority.cases) {
    fields(row, [
      "observation",
      "observationSha256",
      "sourceSha256",
      "facet",
      "historical",
      "historicalSha256",
      "successor",
      "successorSha256",
      "runtime",
    ]);
    const original = read(snapshots, row.historical);
    const next = read(snapshots, row.successor);
    const raw = read(adversarial, row.observation);
    assert.equal(hash(original), row.historicalSha256);
    assert.equal(hash(next), row.successorSha256);
    assert.equal(hash(raw), row.observationSha256);
    verifyAdversarial(
      row,
      snapshotBody(original),
      snapshotBody(next),
      JSON.parse(raw.toString("utf8")),
    );
  }
});

void test("adversarial custody rejects unrelated code, scope, source and runtime mutations", () => {
  const authority = parse<AdversarialAuthority>(adversarial, "custody.json");
  let negatives = 0;
  for (const row of authority.cases) {
    const before = snapshotBody(read(snapshots, row.historical));
    const after = snapshotBody(read(snapshots, row.successor));
    const packet = parse<AdversarialPacket>(adversarial, row.observation);
    for (const mutate of [
      (value: AdversarialPacket) => {
        value.source += "// changed";
      },
      (value: AdversarialPacket) => {
        value[row.facet] += "/* foreign code */";
      },
    ]) {
      const changed = structuredClone(packet);
      mutate(changed);
      assert.throws(() => verifyAdversarial(row, before, after, changed), assert.AssertionError);
      negatives++;
    }
    assert.throws(
      () => verifyAdversarial({ ...row, runtime: true }, before, after, packet),
      assert.AssertionError,
    );
    assert.throws(
      () => verifyAdversarial(row, before, after + "foreign", packet),
      assert.AssertionError,
    );
    negatives += 2;
  }
  assert.equal(negatives, 12);
});

void test("static-value successor retains every other refusal packet facet and rejects mutations", () => {
  assert.equal(
    hash(read(nested, "static-values-10701-successor.json")),
    "49042832cef1a015e79b76bc1106dbc6f7833924bcaba8e712df08b8f1f5ab5d",
  );
  const authority = parse<StaticAuthority>(nested, "static-values-10701-successor.json");
  assert.equal(authority.source, "AfsStepperDialog.stories.tsx.txt");
  assert.equal(hash(read(nested, authority.source)), authority.sourceSha256);
  assert.equal(hash(read(nested, authority.historical)), authority.historicalSha256);
  assert.equal(hash(read(nested, authority.successor)), authority.successorSha256);
  const previous = parse<ModulePacket>(nested, authority.historical);
  const current = parse<ModulePacket>(nested, authority.successor);
  verifyStaticSuccessor(authority, previous, current);
  let mutations = 0;
  for (const mutate of [
    (packet: ModulePacket) => {
      packet.components[0].code += "foreign";
    },
    (packet: ModulePacket) => {
      packet.components[3].map = "foreign";
    },
    (packet: ModulePacket) => {
      packet.components[4].setup = "foreign";
    },
    (packet: ModulePacket) => {
      packet.module = "executable";
    },
    (packet: ModulePacket) => {
      packet.moduleMap = "foreign";
    },
    (packet: ModulePacket) => {
      packet.diagnostics = "[]";
    },
  ]) {
    const changed = structuredClone(current);
    mutate(changed);
    assert.throws(() => verifyStaticSuccessor(authority, previous, changed), assert.AssertionError);
    mutations++;
  }
  assert.throws(
    () =>
      verifyStaticSuccessor(
        { ...authority, allowedChangedPointers: ["/components"] },
        previous,
        current,
      ),
    assert.AssertionError,
  );
  assert.throws(
    () =>
      verifyStaticSuccessor(
        { ...authority, qualification: { ...authority.qualification, runtime: true } },
        previous,
        current,
      ),
    assert.AssertionError,
  );
  assert.equal(mutations, 6);
});

void test("the same whole-packet custody rejects unreviewed mutations independently of hashes", () => {
  const mutations: ((packet: ModulePacket) => unknown)[] = [
    (packet) => delete packet.moduleMap,
    (packet) => (packet.unexpected = true),
    (packet) => (packet.components[0].code += "\n// changed producer"),
    (packet) => (packet.components[0].map = "{}"),
    (packet) => (packet.components[0].style = "changed style"),
    (packet) => (packet.components[0].setup = "changed setup"),
    (packet) => (packet.components[0].name = "changed owner"),
    (packet) => packet.components.pop(),
    (packet) => (packet.diagnostics += ", unexpected diagnostic"),
    (packet) => (packet.module = "export function render() {}"),
  ];
  for (const [index, row] of manifest.cases.entries()) {
    const historical = parse<ModulePacket>(nested, row.historicalPacket);
    const successor = parse<ModulePacket>(nested, row.successorPacket);
    const source = read(nested, row.input);
    for (const mutate of mutations) {
      const changed = structuredClone(successor);
      mutate(changed);
      assert.throws(() => verifySuccessor(specs[index], row, historical, changed, source));
    }
    const broadened = { ...row, allowedChangedFacets: [...row.allowedChangedFacets, "components"] };
    assert.throws(() => verifySuccessor(specs[index], broadened, historical, successor, source));
  }
});

void test("falsy roles preserve historical attribution and distinguish producer and module credit", () => {
  const roles = parse<RoleManifest>(falsy, "falsy-and.authored-module-roles-10701.json");
  fields(roles, [
    "schemaVersion",
    "pullRequest",
    "historical",
    "capture",
    "producerControl",
    "runtimeControl",
  ]);
  assert.equal(roles.schemaVersion, 1);
  assert.equal(roles.pullRequest, manifest.pullRequest);
  const legacy = parse<LegacyRoles>(falsy, "falsy-and.fixed-legacy.json");
  assert.deepEqual(roles.historical, {
    packet: "falsy-and.fixed-legacy.json",
    sha256: "6407caced54e618f9a17081770e9144cfe3a1b88b6bfc991863d3106f9f81e74",
    fixedProducer: legacy.fixedProducer,
    native: legacy.native,
  });
  assert.equal(hash(read(falsy, roles.historical.packet)), roles.historical.sha256);
  assert.equal(roles.historical.fixedProducer.entry, "compile_jsx -> module_code");
  assert.deepEqual(roles.historical.native, { handled: 0, equivalent: 0, pair: "pending" });
  assert.deepEqual(roles.capture, {
    ...provenance,
    packets: [
      {
        packet: "falsy-producers.json",
        sha256: "b281db0ab3f09675e61dd69fdefcf3e4c8bb982fc5ba3b07ea52d96142be7489",
      },
      {
        packet: "falsy-original-refusals.json",
        sha256: "a96434b7ef4c2f263b805b880700439f96e9b722c4327d8f099dc2880d3f3d30",
      },
      {
        packet: "falsy-modules.json",
        sha256: "a8a48d0ab0af5fded0b73739bcb04aa1f071fd33b9d2fc2f8494308e060a1710",
      },
      {
        packet: "falsy-runtime.json",
        sha256: "f9d07c5fc3d863550e5af0724eccf8b415b6ddb773f710c691f96b90acbf1cce",
      },
    ],
  });
  assert.deepEqual(roles.producerControl, {
    scope: "complete historical fixture byte regression; no native authored-module credit",
    backends: [
      {
        backend: "vdom",
        entry: "compile_jsx -> module_code",
        role: "existing authored functional VDOM module",
      },
      {
        backend: "vapor",
        entry: "compile_to_vapor -> components[0].code",
        role: "standalone Vapor renderer producer",
      },
      {
        backend: "ssr",
        entry: "compile_to_ssr -> components[0].code",
        role: "standalone SSR renderer producer",
      },
    ],
    historicalFixedProducerAttributionChanged: false,
  });
  assert.deepEqual(roles.runtimeControl, {
    entry: "App",
    sources: "unchanged original fixture sources",
    clientVapor: {
      defaultMode: "vapor",
      ssr: false,
      role: "authored Vapor component object",
      marker: "__vapor",
      api: "Vue.createVaporApp",
    },
    server: {
      defaultMode: "vapor",
      ssr: true,
      role: "authored SSR Vapor component object",
      markers: ["__ssrInlineRender", "__vapor"],
      api: "vue/server-renderer.renderToString",
    },
    hydration: {
      clientMode: "vdom",
      role: "same-source VDOM client hydrates SSR Vapor-object HTML",
      fixtures: ["boolean", "nested", "values"],
      observations: 42,
    },
    originalFunctionalVdomSsr: {
      defaultMode: "vdom",
      ssr: true,
      role: "complete module explicitly refused; producer remains inspectable",
      fixtures: 4,
    },
    runtimeFailures: 0,
    nativeBytePair: "pending",
    versions: { vue: "3.6.0-rc.9", babel: "7.29.0", babelJsx: "2.0.1" },
    physicalProviderHashes: "not recorded in this receipt",
    releaseQualification: false,
    publicConsumerQualification: false,
  });
});
