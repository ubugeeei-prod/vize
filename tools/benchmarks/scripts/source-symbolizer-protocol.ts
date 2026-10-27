/** Parse only explicit symbolizer/report evidence; missing values never become zero. */
export function sentinelStyle(output: string): "gnu" | "llvm-echo" | null {
  const lines = output.replaceAll("\r\n", "\n").split("\n");
  if (lines[0] === ",") return "llvm-echo";
  if (/^0x0+$/iu.test(lines[0] ?? "") && lines[1] === "??" && lines[2] === "??:0") return "gnu";
  return null;
}

export type SourceCpuRow = {
  dso: string;
  symbol: string;
  source: string;
  selfPercent: number | null;
  childrenPercent: number | null;
  samples: number | null;
  period: string | null;
};

const percentage = (text: string | undefined) => {
  const value = text?.trim().replace(/%$/u, "") ?? "";
  return /^\d+(?:\.\d+)?$/u.test(value) ? Number(value) : null;
};
const count = (text: string | undefined) =>
  text !== undefined && /^\d+$/u.test(text.trim()) && Number.isSafeInteger(Number(text))
    ? Number(text)
    : null;

export function parseFlatSourceReport(output: string) {
  const lines = output.split("\n");
  const header = lines.findIndex(
    (line) =>
      line.startsWith("#") &&
      line.includes("\t") &&
      /(?:Source:Line|srcline)/iu.test(line) &&
      /(?:Overhead|Self)/u.test(line),
  );
  if (header < 0)
    return {
      qualifiedFormat: false,
      reason: "explicit tab-separated report header missing",
      rows: [] as SourceCpuRow[],
    };
  const fields = lines[header]
    .replace(/^#\s*/u, "")
    .split("\t")
    .map((part) => part.trim());
  const index = (pattern: RegExp) => fields.findIndex((field) => pattern.test(field));
  const dso = index(/^(?:Shared Object|DSO)$/iu),
    symbol = index(/^Symbol$/iu);
  const source = index(/^(?:Source:Line|srcline)$/iu),
    self = index(/^(?:Overhead|Self)$/iu);
  const children = index(/^Children$/iu),
    samples = index(/^(?:Samples|Sample)$/iu);
  const period = index(/^(?:Period|Total Period)$/iu);
  if ([dso, symbol, source, self, samples].some((column) => column < 0))
    return {
      qualifiedFormat: false,
      reason: "required self/sample/source columns missing",
      rows: [] as SourceCpuRow[],
    };
  const rows: SourceCpuRow[] = [];
  for (const line of lines.slice(header + 1)) {
    if (!line.trim() || line.startsWith("#")) continue;
    const values = line.split("\t").map((part) => part.trim());
    const selfPercent = percentage(values[self]);
    if (selfPercent === null) continue;
    rows.push({
      dso: values[dso] ?? "",
      symbol: values[symbol] ?? "",
      source: values[source] ?? "",
      selfPercent,
      childrenPercent: children < 0 ? null : percentage(values[children]),
      samples: count(values[samples]),
      period: period < 0 ? null : (values[period] ?? null),
    });
  }
  return {
    qualifiedFormat: rows.length > 0,
    reason: rows.length ? null : "no explicit self rows",
    rows,
  };
}

export function rustSourceCensus(
  rows: SourceCpuRow[],
  targetDso: string,
  totalSamples: number,
  requiredSource?: string,
) {
  const allKnown = rows.every((row) => row.samples !== null);
  const sumRows = (input: SourceCpuRow[]) =>
    input.reduce((total, row) => total + (row.samples ?? 0), 0);
  const allSelfSamples = allKnown ? sumRows(rows) : null;
  const allDsoCountsMatch = allKnown && allSelfSamples === totalSamples;
  const byDso = new Map<string, SourceCpuRow[]>();
  for (const row of rows) {
    const group = byDso.get(row.dso) ?? [];
    group.push(row);
    byDso.set(row.dso, group);
  }
  const allDsos = [...byDso].map(([dso, group]) => {
    const known = group.every((row) => row.samples !== null);
    const resolved = group.filter((row) =>
      /\.(?:rs|go|c|cc|cpp|h|hpp|s|S):[1-9]\d*(?:\b|$)/u.test(row.source),
    );
    const samples = known ? sumRows(group) : null;
    const resolvedSourceSamples = known ? sumRows(resolved) : null;
    return {
      dso,
      samples,
      resolvedSourceSamples,
      unresolvedSourceSamples:
        samples === null || resolvedSourceSamples === null ? null : samples - resolvedSourceSamples,
    };
  });
  const target = rows.filter((row) => row.dso === targetDso);
  const knownCounts = target.every((row) => row.samples !== null);
  const sampled = target.filter((row) => /\.rs:[1-9]\d*(?:\b|$)/u.test(row.source));
  const sum = (input: SourceCpuRow[]) =>
    input.reduce((total, row) => total + (row.samples ?? 0), 0);
  const targetSamples = knownCounts ? sum(target) : null;
  const rustSourceSamples = knownCounts ? sum(sampled) : null;
  const ownsRequestedSource =
    !requiredSource || sampled.some((row) => row.source.includes(requiredSource + ":"));
  const qualified =
    allDsoCountsMatch &&
    Number.isSafeInteger(totalSamples) &&
    totalSamples > 0 &&
    knownCounts &&
    targetSamples !== null &&
    targetSamples <= totalSamples &&
    rustSourceSamples !== null &&
    rustSourceSamples > 0 &&
    ownsRequestedSource;
  return {
    qualified,
    totalSamples,
    allSelfSamples,
    allDsoCountsMatch,
    allDsos,
    targetSamples,
    rustSourceSamples,
    unresolvedTargetSamples:
      targetSamples === null || rustSourceSamples === null
        ? null
        : targetSamples - rustSourceSamples,
    resolvedRows: sampled,
    scope:
      "self samples for the named DSO only; inclusive percentages must never be added to self counts",
  };
}

/** Bind repository locations to the compiled tree, never the observer checkout. */
export function frozenSourceLocations(rows: SourceCpuRow[], sourceRoot: string, sourceSha: string) {
  if (!/^[0-9a-f]{40}$/u.test(sourceSha)) throw new Error("exact compiled source SHA required");
  const prefix = sourceRoot.replace(/\/+$/u, "") + "/";
  const sources = [...new Set(rows.map((row) => row.source))];
  return sources.map((source) => {
    const match = /^(.*):([1-9]\d*)$/u.exec(source);
    const file = match?.[1],
      line = match ? Number(match[2]) : null;
    const path = file?.startsWith(prefix) ? file.slice(prefix.length) : null;
    const owned =
      path !== null &&
      path.length > 0 &&
      !path.split("/").some((part) => part === "." || part === "..");
    return {
      source,
      line,
      compiledSourceSha: sourceSha,
      repositoryPath: owned ? path : null,
      url: owned
        ? "https://github.com/ubugeeei-prod/vize/blob/" +
          sourceSha +
          "/" +
          path.split("/").map(encodeURIComponent).join("/") +
          "#L" +
          line
        : null,
      scope: owned
        ? "repository at the compiled source SHA"
        : "external toolchain/dependency or unresolved; no current-checkout source link",
    };
  });
}

export function completeKnownFrames(
  output: string,
  addresses: string[],
  style: "gnu" | "llvm-echo" | null,
) {
  if (style === null) return false;
  const lines = output.trimEnd().replaceAll("\r\n", "\n").split("\n");
  const tokens: string[] = [];
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    if (style === "gnu" && /^0x0+$/iu.test(line)) {
      if (lines[index + 1] !== "??" || lines[index + 2] !== "??:0") return false;
      tokens.push("sentinel");
    } else if (style === "llvm-echo" && line === ",") tokens.push("sentinel");
    else if (/^0x[0-9a-f]+$/iu.test(line)) tokens.push(BigInt(line).toString(16));
  }
  const expected = [
    "sentinel",
    ...addresses.flatMap((address) => [BigInt("0x" + address).toString(16), "sentinel"]),
  ];
  return JSON.stringify(tokens) === JSON.stringify(expected);
}

export type SymbolCpuRow = Omit<SourceCpuRow, "source" | "childrenPercent">;
export function parseFlatSymbolReport(output: string) {
  const lines = output.split("\n");
  const header = lines.findIndex(
    (line) =>
      line.startsWith("#") &&
      line.includes("\t") &&
      /(?:Overhead|Self)/u.test(line) &&
      /Symbol/u.test(line),
  );
  if (header < 0) return { qualifiedFormat: false, rows: [] as SymbolCpuRow[] };
  const fields = lines[header]
    .replace(/^#\s*/u, "")
    .split("\t")
    .map((part) => part.trim());
  const index = (pattern: RegExp) => fields.findIndex((field) => pattern.test(field));
  const dso = index(/^(?:Shared Object|DSO)$/iu),
    symbol = index(/^Symbol$/iu),
    self = index(/^(?:Overhead|Self)$/iu),
    samples = index(/^(?:Samples|Sample)$/iu),
    period = index(/^(?:Period|Total Period)$/iu);
  if ([dso, symbol, self, samples].some((column) => column < 0))
    return { qualifiedFormat: false, rows: [] as SymbolCpuRow[] };
  const rows: SymbolCpuRow[] = [];
  for (const line of lines.slice(header + 1)) {
    if (!line.trim() || line.startsWith("#")) continue;
    const values = line.split("\t").map((part) => part.trim()),
      selfPercent = percentage(values[self]);
    if (selfPercent === null) continue;
    rows.push({
      dso: values[dso] ?? "",
      symbol: values[symbol] ?? "",
      selfPercent,
      samples: count(values[samples]),
      period: period < 0 ? null : (values[period] ?? null),
    });
  }
  return { qualifiedFormat: rows.length > 0, rows };
}
