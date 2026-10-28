/** Convert recorded runtime PCs using matching MMAP and executable ELF segments. */
export interface CheckerSample {
  ordinal: number;
  pid: string;
  tid: string;
  timestamp: string;
  period: string;
  runtimePc: string;
  image: string;
  reportedFileOffset: string | null;
}
export interface CheckerMapping {
  pid: string;
  timestamp: string;
  start: string;
  length: string;
  fileOffset: string;
  image: string;
}
const ns = (time: string) => {
  const [seconds, fraction] = time.split(".");
  return BigInt(seconds) * 1000000000n + BigInt(fraction);
};
const hex = (value: bigint) => "0x" + value.toString(16);
export function bindCheckerProgramCounters(
  trace: string,
  elf: string,
  image: string,
  expected = { all: 1826, checker: 1380 },
) {
  const samples: CheckerSample[] = [],
    mappings: CheckerMapping[] = [];
  for (const line of trace.split("\n")) {
    const sample = /^\s*(\d+)\/(\d+)\s+(\d+\.\d{9}):\s+(\d+)\s+([0-9a-f]+)\s+\((.+)\)$/iu.exec(
      line,
    );
    if (sample) {
      const offset = /^(.*)\+0x([0-9a-f]+)$/iu.exec(sample[6]);
      samples.push({
        ordinal: samples.length,
        pid: sample[1],
        tid: sample[2],
        timestamp: sample[3],
        period: sample[4],
        runtimePc: "0x" + sample[5],
        image: offset?.[1] ?? sample[6],
        reportedFileOffset: offset ? "0x" + offset[2] : null,
      });
      continue;
    }
    const map =
      /^\s*(\d+)\/\d+\s+(\d+\.\d{9}): PERF_RECORD_MMAP2 (\d+)\/\d+: \[(0x[0-9a-f]+)\((0x[0-9a-f]+)\) @ (0x[0-9a-f]+|0) [^\]]+\]: ([rwxsp-]+) (.+)$/iu.exec(
        line,
      );
    if (map && map[8] === image && map[7].includes("x")) {
      if (map[1] !== map[3]) throw new Error("MMAP PID header/payload mismatch");
      mappings.push({
        pid: map[3],
        timestamp: map[2],
        start: map[4],
        length: map[5],
        fileOffset: map[6],
        image: map[8],
      });
    }
  }
  if (!/Type:\s+EXEC\b/u.test(elf))
    throw new Error("this frozen checker requires the observed EXEC image");
  const segments = elf.split("\n").flatMap((line) => {
    const match =
      /^\s*LOAD\s+(0x[0-9a-f]+)\s+(0x[0-9a-f]+)\s+0x[0-9a-f]+\s+(0x[0-9a-f]+)\s+0x[0-9a-f]+\s+([RWE ]+)\s+0x[0-9a-f]+\s*$/iu.exec(
        line,
      );
    return match && match[4].includes("E")
      ? [
          {
            fileOffset: match[1],
            virtualAddress: match[2],
            fileSize: match[3],
          },
        ]
      : [];
  });
  const checker = samples.filter((sample) => sample.image === image);
  const problems: { ordinal: number; reason: string }[] = [];
  const rows = checker.flatMap((sample) => {
    const pc = BigInt(sample.runtimePc);
    const candidates = mappings
      .filter(
        (map) =>
          map.pid === sample.pid &&
          ns(map.timestamp) <= ns(sample.timestamp) &&
          pc >= BigInt(map.start) &&
          pc < BigInt(map.start) + BigInt(map.length),
      )
      .sort((left, right) => (ns(left.timestamp) > ns(right.timestamp) ? -1 : 1));
    const map = candidates[0];
    const fail = (reason: string) => {
      problems.push({ ordinal: sample.ordinal, reason });
      return [];
    };
    if (!map) return fail("no prior executable SDK mapping for the sampled PID/PC");
    const fileOffset = pc - BigInt(map.start) + BigInt(map.fileOffset);
    if (sample.reportedFileOffset === null || BigInt(sample.reportedFileOffset) !== fileOffset)
      return fail("independent perf dsoff disagrees with MMAP file offset");
    const matches = segments.filter(
      (segment) =>
        fileOffset >= BigInt(segment.fileOffset) &&
        fileOffset < BigInt(segment.fileOffset) + BigInt(segment.fileSize),
    );
    if (matches.length !== 1) return fail("file offset has no unique executable PT_LOAD");
    const segment = matches[0];
    const elfPc = BigInt(segment.virtualAddress) + fileOffset - BigInt(segment.fileOffset);
    if (pc - elfPc !== 0n) return fail("observed EXEC SDK unexpectedly has nonzero load bias");
    return [
      {
        ...sample,
        elfPc: hex(elfPc),
        fileOffset: hex(fileOffset),
        loadBias: "0x0",
        mmapStart: map.start,
        mmapFileOffset: map.fileOffset,
        segmentFileOffset: segment.fileOffset,
        segmentVirtualAddress: segment.virtualAddress,
      },
    ];
  });
  return {
    qualified:
      samples.length === expected.all &&
      checker.length === expected.checker &&
      checker.length > 0 &&
      rows.length === checker.length &&
      problems.length === 0,
    allSampleCount: samples.length,
    checkerSampleCount: checker.length,
    mappedCheckerSamples: rows.length,
    uniqueElfPcs: [...new Set(rows.map((row) => row.elfPc))],
    mappings,
    executableSegments: segments,
    rows,
    problems,
    conversion:
      "runtime PC - MMAP start + MMAP pgoff -> PT_LOAD virtual PC; independent perf dsoff checked",
    sourceOwnershipVerified: false,
  };
}
