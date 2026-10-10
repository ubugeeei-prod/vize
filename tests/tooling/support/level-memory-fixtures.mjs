import fs from "node:fs";
import { createHash } from "node:crypto";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import * as memory from "../../../tools/benchmarks/scripts/level-memory-lib.mjs";

const repository = fileURLToPath(new URL("../../..", import.meta.url));
export const report = (id) => ({
  bench_id: id,
  fixture: `synthetic:${id}`,
  platform: "linux",
  wall_ns: { p50: 1, p95: 2 },
  allocs: 0,
  alloc_bytes_peak: 0,
  rss_peak_bytes: 0,
  harness_version: "1.0.0",
});
export function fixture(t) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "level-memory-law-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const dir = path.join(root, "tools/benchmarks/results/davinci");
  fs.mkdirSync(dir, { recursive: true });
  const identities = new Map(
    ["one", "two"].map((id) => [
      id,
      {
        bench_id: id,
        fixture: `synthetic:${id}`,
        fixture_sha256: "a".repeat(64),
        window: "routine-return",
      },
    ]),
  );
  for (const id of identities.keys())
    fs.writeFileSync(path.join(dir, `${id}.json`), JSON.stringify(report(id)));
  const markers = [...identities.keys()]
    .map((id) => `davinci-harness: wrote ${path.join(dir, `${id}.json`)}`)
    .join("\n");
  return { root, dir, identities, markers };
}

export function originalPopulation() {
  const authority = memory.loadMemoryAuthority(repository);
  const rows = new Map(
    [...authority.registry].map((id) => [
      id,
      {
        report: report(id),
        identity: {
          bench_id: id,
          fixture: `synthetic:${id}`,
          fixture_sha256: "a".repeat(64),
          window: "routine-return",
        },
      },
    ]),
  );
  return { authority, rows };
}

export function sourceHarnessFixture(t, version, existingRoot) {
  const root =
    existingRoot ??
    fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "memory-source-harness-")));
  if (existingRoot === undefined) t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const directory = path.join(root, "tools/benchmarks/crates/davinci_harness");
  fs.mkdirSync(path.join(directory, "src"), { recursive: true });
  fs.writeFileSync(path.join(root, "Cargo.toml"), `[workspace.package]\nversion = "${version}"\n`);
  const manifest = path.join(directory, "Cargo.toml");
  fs.writeFileSync(manifest, '[package]\nname = "davinci_harness"\nversion.workspace = true\n');
  const source = path.join(directory, "src/lib.rs");
  fs.writeFileSync(source, 'pub const VERSION: &str = env!("CARGO_PKG_VERSION");\n');
  const sourcePaths = [
    "Cargo.toml",
    "tools/benchmarks/crates/davinci_harness/Cargo.toml",
    "tools/benchmarks/crates/davinci_harness/src/lib.rs",
  ];
  const wholeTree = sourcePaths
    .map((relative) => {
      const bytes = fs.readFileSync(path.join(root, relative));
      const blob = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
      return `100644 blob ${blob}\t${relative}\n`;
    })
    .join("");
  const id = `path+file://${directory}#${version}`;
  return {
    root,
    wholeTree,
    metadata: {
      workspace_root: root,
      packages: [{ name: "davinci_harness", version, id, manifest_path: manifest }],
    },
    artifacts: [
      {
        reason: "compiler-artifact",
        package_id: id,
        manifest_path: manifest,
        target: { kind: ["lib"], src_path: source },
      },
    ],
  };
}
