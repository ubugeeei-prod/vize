import crypto from "node:crypto";
import { parse as parseToml } from "@iarna/toml";
import {
  core,
  host,
  exportFile,
  exportTests,
  removedExports,
  hostDeclaration,
  callerChanges,
  calls,
  edges,
  hashes,
  gateChanges,
} from "./profile-export-host-contract.ts";

export const compact = (text: string) => text.replace(/\s+/gu, "").replace(/,(?=[}\]])/gu, "");
export const digest = (text: string) => crypto.createHash("sha256").update(text).digest("hex");
type Reader = (file: string) => string;
type Table = Record<string, unknown>;
const table = (value: unknown): value is Table =>
  !!value && typeof value === "object" && !Array.isArray(value);
const requireState = (ok: boolean, message: string) => {
  if (!ok) throw new Error(message);
};
const occurrences = (source: string, text: string) =>
  compact(source).split(compact(text)).length - 1;

export function validateEdges(read: Reader, old: boolean) {
  for (const [name, file, kind, declaration] of edges) {
    const manifest = parseToml(read(file));
    const found: { key: string; kind: string; value: unknown }[] = [];
    function visit(value: Table, prefix: string) {
      for (const [key, entry] of Object.entries(value)) {
        if (!table(entry)) continue;
        if (["dependencies", "dev-dependencies", "build-dependencies"].includes(key)) {
          for (const [alias, dependency] of Object.entries(entry))
            if (
              alias === "vize_carton" ||
              (table(dependency) && dependency.package === "vize_carton")
            )
              found.push({ key: alias, kind: prefix + key, value: dependency });
        } else visit(entry, prefix + key + ".");
      }
    }
    visit(manifest, "");
    const expected = parseToml(`[${kind}]\n${declaration}\n`)[kind] as Table;
    requireState(
      old
        ? found.length === 0
        : found.length === 1 &&
            found[0].key === "vize_carton" &&
            found[0].kind === kind &&
            JSON.stringify(found[0].value) === JSON.stringify(expected.vize_carton),
      `unexpected profile host dependency: ${name}`,
    );
  }
  for (const file of ["davinci/vize_l0/Cargo.toml", "crates/vize_carton/Cargo.toml"])
    requireState(
      (parseToml(read(file)).package as Table).version !== undefined &&
        table((parseToml(read(file)).package as Table).version) &&
        ((parseToml(read(file)).package as Table).version as Table).workspace === true,
      "profile version must retain workspace identity",
    );
}

export function prepare(read: Reader): Map<string, string> {
  const planned = new Map<string, string>();
  const get = (file: string) => planned.get(file) ?? read(file);
  const coreSource = read(core),
    hostSource = read(host);
  const old =
    occurrences(coreSource, "mod export;") === 1 &&
    occurrences(coreSource, removedExports) === 1 &&
    !hostSource.includes("pub mod profile_export;");
  const next =
    !/\bmod export;|\bProfileExport/u.test(coreSource) &&
    !hostSource.includes(
      "host profile JSON preserves its existing owned span and counter vectors",
    ) &&
    occurrences(hostSource, hostDeclaration) === 1;
  requireState(old || next, "unexpected profile export module ownership");
  const sfcGate = digest(read("tests/tooling/davinci/davinci-atelier-sfc-stage-alias.test.mjs"));
  requireState(
    old ? sfcGate === hashes.sfcGate[0] : hashes.sfcGate[1].includes(sfcGate),
    "unexpected exact SFC dependency gate",
  );
  requireState(
    digest(read("davinci/vize_l0/src/profiler/snapshot.rs")) === hashes.provider,
    "unexpected metric readback body",
  );
  requireState(
    (old
      ? digest(read(exportFile)) === hashes.exporter[0]
      : hashes.exporter[1].includes(digest(read(exportFile)))) &&
      (old
        ? digest(read(exportTests)) === hashes.tests[0]
        : hashes.tests[1].includes(digest(read(exportTests)))),
    "unexpected moved profile exporter or wire law",
  );
  if (read(exportFile).includes("mod assemble;"))
    requireState(
      digest(read(hashes.assembly[0])) === hashes.assembly[1],
      "unexpected exact profile assembly body",
    );
  requireState(
    read("davinci/vize_l0/src/profiler/snapshot.rs").includes("pub fn span_snapshot(&self)") &&
      read("davinci/vize_l0/src/profiler/snapshot.rs").includes("pub fn counter_snapshot(&self)"),
    "public metric readback provider is required",
  );
  validateEdges(read, old);
  const ownedCaller = !old && digest(get(hashes.snapshotCaller[0])) === hashes.snapshotCaller[1];
  const companion = digest(read("tests/tooling/support/davinci-profile-host-imports.ts"));
  requireState(
    ownedCaller
      ? companion === hashes.snapshotCompanion
      : companion === hashes.companion || (!old && companion === hashes.snapshotCompanion),
    "the reviewed profile host-import companion must accompany replay",
  );
  function change(file: string, before: string, after: string, count = 1) {
    const source = get(file);
    const afterParts = after.startsWith("use ") ? after.split(/(?<=;)\s*(?=use )/u) : [after];
    requireState(
      old
        ? occurrences(source, before) === count &&
            (!after || afterParts.every((part) => occurrences(source, part) === 0))
        : (after
            ? afterParts.every((part) => occurrences(source, part) === count)
            : occurrences(source, before) === 0) &&
            (!before || after.includes(before) || occurrences(source, before) === 0),
      "unexpected profile integration state: " + file,
    );
    if (old) {
      requireState(
        source.split(before).length - 1 === count,
        "unexpected profile replay formatting: " + file,
      );
      planned.set(file, source.replaceAll(before, after));
    }
  }
  change(core, "mod export;\n", "");
  change(core, removedExports, "");
  change("davinci/vize_l0/src/profiler/tests.rs", "mod export;\n\n", "");
  for (const [file, before, after] of gateChanges) {
    const source = get(file);
    if (file !== "tests/tooling/davinci-stage-dependencies.test.ts") {
      const count = (text: string) => source.split(text).length - 1;
      requireState(
        old
          ? count(before) === 1 && count(after) === 0
          : count(after) === 1 && (after.includes(before) || count(before) === 0),
        "unexpected exact profile gate wiring",
      );
      if (old) planned.set(file, source.replace(before, after));
    } else {
      const matches = [...source.matchAll(/hostRuntime:\s*(\[[^\]]*\])\.includes\(/gu)];
      requireState(matches.length === 1, "unexpected profile hostRuntime gate");
      const list = JSON.parse(matches[0][1].replace(/,\s*\]$/u, "]"));
      requireState(
        JSON.stringify(list) === JSON.stringify(JSON.parse(old ? before : after)),
        "unexpected exact profile hostRuntime gate",
      );
      if (old) planned.set(file, source.replace(matches[0][1], after));
    }
  }
  if (old) {
    requireState(hostSource.includes("pub mod config;\n"), "missing host insertion anchor");
    planned.set(
      host,
      hostSource.replace("pub mod config;\n", "pub mod config;\n\n" + hostDeclaration),
    );
  }
  const oldImports = `use crate::String;\n\nuse super::allocation::AllocationSnapshot;\nuse super::attribution::SpanAttribution;\nuse super::core::Profiler;\nuse super::metrics::Metrics;`;
  const newImports = `use vize_l0::String;\nuse vize_l0::profiler::{AllocationSnapshot, Metrics, Profiler, SpanAttribution};`;
  // The complete reviewed owned-input facade and assembly were qualified above.
  // Its additional CounterMetrics import and API link are not old move inputs.
  if (!read(exportFile).includes("mod assemble;")) {
    change(exportFile, oldImports, newImports);
    change(exportFile, "[`Profiler::export_report`]", "[`export_report`]");
  }
  const exporter = get(exportFile);
  if (old) {
    const start = exporter.indexOf("impl Profiler {\n"),
      end = exporter.indexOf("\n}\n\nfn span_entry", start);
    requireState(start >= 0 && end > start, "unexpected profile exporter function");
    const method = exporter
      .slice(start + "impl Profiler {\n".length, end)
      .replace(/^    /gm, "")
      .replace(
        "pub fn export_report(&self, options:",
        "pub fn export_report(profiler: &Profiler, options:",
      )
      .replaceAll("self.span_snapshot()", "profiler.span_snapshot()")
      .replaceAll("self.counter_snapshot()", "profiler.counter_snapshot()");
    requireState(
      !method.includes("&self") &&
        method.includes("profiler.span_snapshot()") &&
        method.includes("profiler.counter_snapshot()"),
      "unexpected profile exporter receiver",
    );
    planned.set(
      exportFile,
      exporter.slice(0, start) + method + exporter.slice(end + 2) + "\n#[cfg(test)]\nmod tests;\n",
    );
  } else
    requireState(
      occurrences(exporter, "pub fn export_report(profiler: &Profiler,") === 1 &&
        !exporter.includes("impl Profiler") &&
        occurrences(exporter, "mod tests;") === 1,
      "host exporter must consume the readbacks directly",
    );
  change(
    exportTests,
    `use super::super::{\n    ProfileExportBudget, ProfileExportOptions, Profiler, SpanAttribution, SpanRange,\n};`,
    `use super::{ProfileExportBudget, ProfileExportOptions, export_report};\nuse vize_l0::profiler::{Profiler, SpanAttribution, SpanRange};`,
  );
  change(
    exportTests,
    "use crate::profiler::AllocationSnapshot;",
    "use vize_l0::profiler::AllocationSnapshot;",
  );
  for (const [file, before, after] of callerChanges)
    if (!ownedCaller || file !== hashes.snapshotCaller[0]) change(file, before, after);
  for (const [file, before, after] of calls) {
    requireState(
      (get(file).match(/\.export_report\s*\(/gu) ?? []).length ===
        (old ? (file === exportTests ? 6 : 1) : 0),
      "unexpected extra profiler export call: " + file,
    );
    if (!ownedCaller || file !== hashes.snapshotCaller[0])
      change(file, before, after, file === exportTests ? 6 : 1);
    requireState(
      !/vize_l0\s*::\s*profiler\s*::\s*(?:ProfileExport|PROFILE_EXPORT_SCHEMA_VERSION|\{[^}]*\bProfileExport)/u.test(
        get(file),
      ),
      "retired core profile wire import: " + file,
    );
  }
  if (old)
    for (const [, file, kind, declaration] of edges) {
      const source = get(file),
        anchor = `[${kind}]\n`;
      requireState(source.split(anchor).length === 2, "missing profile manifest insertion anchor");
      planned.set(file, source.replace(anchor, anchor + declaration + "\n"));
    }
  let lock = read("Cargo.lock");
  const parsed = parseToml(lock).package;
  requireState(Array.isArray(parsed), "malformed profile lock packages");
  for (const [name] of edges) {
    const packages = (parsed as Table[]).filter((entry) => entry.name === name);
    requireState(
      packages.length === 1 && Array.isArray(packages[0].dependencies),
      "missing profile lock package: " + name,
    );
    const dependencies = packages[0].dependencies as string[];
    requireState(
      old
        ? !dependencies.includes("vize_carton")
        : dependencies.filter((edge) => edge === "vize_carton").length === 1,
      "unexpected profile lock ownership: " + name,
    );
    if (old) {
      const start = lock.indexOf(`name = "${name}"\n`),
        end = lock.indexOf("\n[[package]]", start);
      requireState(start >= 0 && end > start, "unexpected profile lock package layout");
      const entry = lock.slice(start, end),
        list = entry.match(/dependencies = \[\n([\s\S]*?)\n\]/u);
      requireState(!!list, "unexpected profile lock dependency layout");
      const lines = [...list![1].split("\n"), ' "vize_carton",'].sort();
      lock = lock.replace(
        entry,
        entry.replace(list![0], "dependencies = [\n" + lines.join("\n") + "\n]"),
      );
    }
  }
  if (old) planned.set("Cargo.lock", lock);
  for (const [, after] of edges)
    if (old) requireState(planned.has(after), "missing profile host dependency write");
  return planned;
}
