import { relative } from "node:path";

type TestEvent = {
  type: string;
  data?: { file?: string; nesting?: number; details?: { duration_ms?: number } };
};

// A second `node --test` reporter beside the unchanged spec output. Each file runs in its own
// process, and the runner reports that process as a top-level test of the same file. The
// longest top-level completion per file is therefore its wall time, imports included.
export default async function* toolingTestTimingReporter(
  source: AsyncIterable<TestEvent>,
): AsyncGenerator<string> {
  const files: Record<string, number> = {};
  for await (const { type, data } of source) {
    if (type !== "test:complete" || data?.nesting !== 0 || !data.file) continue;
    const file = relative(process.cwd(), data.file).split("\\").join("/");
    const ms = Math.round(data.details?.duration_ms ?? 0);
    files[file] = Math.max(files[file] ?? 0, ms);
  }
  yield `${JSON.stringify(files)}\n`;
}
