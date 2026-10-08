/** Exact filesystem reads at their actual host callers; no storage allowance. */
export const sourceIoDirectReads: Record<string, number> = {
  "crates/vize/src/commands/build/runner/compile.rs": 3,
  "crates/vize/src/commands/build/runner/compile_stats.rs": 1,
  "crates/vize_vitrine/src/napi/sfc/batch.rs": 1,
};

export const sourceIoAliasReads = [
  "crates/vize/src/commands/fmt.rs",
  "crates/vize/src/commands/lint.rs",
  "crates/vize_maestro/src/server/state/global_tag_names.rs",
];

export function withoutSourceIoHostReferences(source: string, file: string): string {
  const direct = /\bvize_carton::source_io::read_to_string\b(?=\s*\()/gu;
  const expected = sourceIoDirectReads[file];
  if (expected && [...source.matchAll(direct)].length === expected) {
    return source.replace(direct, "host_source_read");
  }
  if (sourceIoAliasReads.includes(file)) {
    const declaration = /^use vize_carton::source_io as fs;$/gmu;
    const uses = [...source.matchAll(/(?<!::)\bfs\s*::/gu)];
    if (
      [...source.matchAll(declaration)].length === 1 &&
      uses.length === 1 &&
      [...source.matchAll(/(?<!::)\bfs::read_to_string\s*\(/gu)].length === 1
    ) {
      return source.replace(declaration, "host_source_read_import");
    }
  }
  return source;
}
