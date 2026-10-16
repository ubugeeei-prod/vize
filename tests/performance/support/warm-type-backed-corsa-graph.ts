import assert from "node:assert/strict";
import TOML from "@iarna/toml";

export const corsaPackages = [
  "corsa",
  "corsa_core",
  "corsa_jsonrpc",
  "corsa_client",
  "corsa_lsp",
  "corsa_orchestrator",
] as const;
const consumers = ["vize", "vize_canon", "vize_patina"];
type Manifest = {
  workspace: {
    package: { version: string };
    members: string[];
    dependencies: Record<string, unknown>;
  };
};
type Package = {
  name: string;
  version: string;
  source?: string;
  checksum?: string;
  dependencies?: string[];
};
type Lock = { package: Package[] };

/** Only the six named registry-to-owned identities and their exact consumer edges may migrate. */
export function qualifyCorsaGraph(
  beforeManifest: string,
  afterManifest: string,
  beforeLock: string,
  afterLock: string,
) {
  const before = TOML.parse(beforeManifest) as unknown as Manifest;
  const after = TOML.parse(afterManifest) as unknown as Manifest;
  const baseline = TOML.parse(beforeLock) as unknown as Lock;
  const current = TOML.parse(afterLock) as unknown as Lock;
  const unique = (records: Package[], name: string): Package => {
    const selected = records.filter((entry) => entry.name === name);
    assert.equal(selected.length, 1, `one exact ${name} record is required`);
    return selected[0];
  };
  const version = after.workspace.package.version;
  assert.equal(typeof version, "string");
  const expectedManifest = structuredClone(before);
  const members = corsaPackages.map((name) => `vendor/${name}`);
  assert.ok(members.every((name) => !before.workspace.members.includes(name)));
  assert.deepEqual(
    after.workspace.members.toSorted(),
    [...before.workspace.members, ...members].toSorted(),
  );
  expectedManifest.workspace.members = after.workspace.members;
  const dependencies = expectedManifest.workspace.dependencies;
  for (const name of corsaPackages) {
    const original = unique(baseline.package, name);
    assert.match(original.source ?? "", /^registry\+/u);
    assert.match(original.checksum ?? "", /^[a-f0-9]{64}$/u);
    if (name === "corsa" || name === "corsa_lsp")
      assert.deepEqual(dependencies[name], { package: name, version: `=${original.version}` });
    else assert.equal(dependencies[name], undefined);
    dependencies[name] = {
      package: `vize_${name}`,
      version: `=${version}`,
      path: `vendor/${name}`,
    };
  }
  assert.deepEqual(dependencies.corsa_runtime, { package: "corsa_runtime", version: "=1.14.0" });
  dependencies.corsa_runtime = { package: "corsa_runtime", version: "=1.14.2" };
  assert.deepEqual(
    after,
    expectedManifest,
    "only six workspace aliases and the declared runtime pin may change",
  );
  const runtime = unique(baseline.package, "corsa_runtime");
  assert.equal(runtime.version, "1.14.2");
  assert.deepEqual(
    unique(current.package, "corsa_runtime"),
    runtime,
    "the actual native runtime must remain whole and identical",
  );
  const owned = (edge: string) => {
    const [name, ...suffix] = edge.split(" ");
    return [
      ...(corsaPackages.includes(name as (typeof corsaPackages)[number])
        ? [`vize_${name}`]
        : [name]),
      ...suffix,
    ].join(" ");
  };
  const expectedLock = structuredClone(baseline);
  for (const record of expectedLock.package) {
    if (corsaPackages.includes(record.name as (typeof corsaPackages)[number])) {
      const originalName = record.name;
      record.name = `vize_${originalName}`;
      record.version = version;
      delete record.source;
      delete record.checksum;
      assert.ok(record.dependencies);
      record.dependencies = record.dependencies.map(owned);
      // Workspace membership records retained dev/optional edges even when the CLI does not enable them.
      if (originalName === "corsa") record.dependencies.push("tempfile");
      if (originalName === "corsa_orchestrator") record.dependencies.push("lsp-types 0.97.0");
      record.dependencies.sort();
    } else if (consumers.includes(record.name)) {
      assert.ok(record.dependencies);
      assert.ok(
        record.dependencies.some((edge) => owned(edge) !== edge),
        "each named consumer must replace actual provider edges",
      );
      record.dependencies = record.dependencies.map(owned).sort();
    }
  }
  for (const name of consumers) {
    unique(baseline.package, name);
    unique(current.package, name);
  }
  const order = (lock: Lock) => ({
    ...lock,
    package: lock.package.toSorted((a, b) =>
      JSON.stringify([a.name, a.version, a.source]).localeCompare(
        JSON.stringify([b.name, b.version, b.source]),
      ),
    ),
  });
  assert.deepEqual(
    order(current),
    order(expectedLock),
    "all unrelated lock records and external edges must remain whole",
  );
  return {
    providers: [...corsaPackages],
    consumers,
    unchangedRecords: baseline.package.length - corsaPackages.length - consumers.length,
    runtime,
  };
}
