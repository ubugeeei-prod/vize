import { nodeTestCases, packet, type NodeTestEvent } from "./test-cases.ts";

// A `node --test` reporter that writes this job's test case packet. It counts what
// Node's own summary counts as tests: every test, nested ones included, never suites.
// A file that fails to load is reported as a failed test named after the file.
// `VIZE_TEST_CASES_ROOT` makes paths repository-relative when the runner's cwd is a package.
export default async function* testCaseReporter(
  source: AsyncIterable<NodeTestEvent>,
): AsyncGenerator<string> {
  const events: NodeTestEvent[] = [];
  for await (const event of source) events.push(event);
  const cases = nodeTestCases(events, process.env.VIZE_TEST_CASES_ROOT || process.cwd());
  yield `${JSON.stringify(packet(process.env, cases))}\n`;
}
