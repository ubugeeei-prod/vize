/** Read literal metadata or a file-local string constant without executing Rust. */
export function rustMetadataString(block: string, field: string, source: string): string | null {
  const literal = '"((?:\\\\.|[^"\\\\])*)"';
  const quoted = block.match(new RegExp(`\\b${field}:\\s*${literal}`));
  if (quoted) return JSON.parse(`"${quoted[1]}"`) as string;

  const identifier = block.match(new RegExp(`\\b${field}:\\s*([A-Z][A-Z0-9_]*)\\s*,`))?.[1];
  if (!identifier) return null;
  const constant = source.match(
    new RegExp(`\\bconst\\s+${identifier}\\s*:\\s*&(?:'static\\s+)?str\\s*=\\s*${literal}\\s*;`),
  );
  if (!constant) throw new Error(`Unresolved metadata string: ${field}: ${identifier}`);
  return JSON.parse(`"${constant[1]}"`) as string;
}
