import { relative } from "node:path";
import { packet, type Outcome, type TestCase } from "./test-cases.ts";

type TestEvent = {
  type: string;
  data?: {
    name?: string;
    file?: string;
    skip?: boolean | string;
    todo?: boolean | string;
    details?: { type?: string };
  };
};

// A `node --test` reporter that writes this job's test case packet. It counts what
// Node's own summary counts as tests: every test, nested ones included, never suites.
// A file that fails to load is reported as a failed test named after the file.
export default async function* testCaseReporter(
  source: AsyncIterable<TestEvent>,
): AsyncGenerator<string> {
  const cases: TestCase[] = [];
  for await (const { type, data } of source) {
    if ((type !== "test:pass" && type !== "test:fail") || data?.details?.type !== "test") continue;
    if (!data.file || !data.name) throw new Error(`A ${type} event has no file or name`);
    const file = relative(process.cwd(), data.file).split("\\").join("/");
    const outcome: Outcome =
      data.skip || data.todo ? "skipped" : type === "test:fail" ? "failed" : "passed";
    const name = data.name === file ? file : `${file} › ${data.name}`;
    cases.push({ path: file, name, outcome });
  }
  yield `${JSON.stringify(packet(process.env, cases))}\n`;
}
