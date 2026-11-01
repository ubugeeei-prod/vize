import assert from "node:assert/strict";

export const ORDER = ["baseline", "candidate", "candidate", "baseline"] as const;
export const IMAGE = "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70";
export const SELECTOR =
  "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::shared_contexts::production::resource::native_two_config_owner_resource_discriminator";
const MEMORY_LIMIT_KIB = 2 * 1024 * 1024;
export const object = (value: unknown): Record<string, unknown> => {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
};

function stat(value: unknown): { pid: number; ppid: number; birth: number; state: string } {
  assert.equal(typeof value, "string");
  const text = value as string;
  const split = text.lastIndexOf(") ");
  assert.ok(split > 2);
  const pid = Number(text.slice(0, text.indexOf(" (")));
  assert.ok(Number.isSafeInteger(pid) && pid > 0);
  const fields = text
    .slice(split + 2)
    .trim()
    .split(/\s+/);
  assert.ok(fields.length >= 22);
  const state = fields[0]!;
  assert.match(state, /^[A-Z]$/);
  assert.notEqual(state, "Z");
  const ppid = Number(fields[1]),
    birth = Number(fields[19]);
  assert.ok(Number.isSafeInteger(ppid) && ppid >= 0 && Number.isSafeInteger(birth) && birth > 0);
  return { pid, ppid, birth, state };
}
function kib(value: unknown, field: string): number {
  assert.equal(typeof value, "string");
  const lines = (value as string).split("\n").filter((line) => line.startsWith(field + ":"));
  assert.equal(lines.length, 1);
  const match = new RegExp(`^${field}:\\s+([0-9]+) kB$`).exec(lines[0]!);
  assert.ok(match);
  const result = Number(match[1]);
  assert.ok(Number.isSafeInteger(result) && result >= 0);
  return result;
}
export function memory(
  row: Record<string, unknown>,
  expectedNative: number,
  executable: string,
  testHash: string,
  native: string,
  input: string,
  expectedPid: number,
  sourceRoot: string,
): void {
  assert.ok(Array.isArray(row.rows));
  const members = row.rows.map(object);
  assert.equal(members.length, expectedNative + 1);
  assert.equal(row.processCount, members.length);
  assert.ok(Number(row.collectionEndUnixUs) >= Number(row.collectionStartUnixUs));
  let rss = 0,
    pss = 0;
  const ids = new Set<number>();
  let rootPid = 0;
  for (const [index, member] of members.entries()) {
    const before = stat(member.statBefore),
      after = stat(member.statAfter);
    assert.equal(before.pid, after.pid);
    assert.equal(before.birth, after.birth);
    assert.equal(before.ppid, after.ppid);
    assert.equal(member.pid, before.pid);
    assert.equal(member.birth, before.birth);
    assert.ok(!ids.has(before.pid));
    ids.add(before.pid);
    assert.ok(Number(member.collectionStartUnixUs) >= Number(row.collectionStartUnixUs));
    assert.ok(Number(member.collectionEndUnixUs) <= Number(row.collectionEndUnixUs));
    const rollupRss = kib(member.smapsRollup, "Rss"),
      rollupPss = kib(member.smapsRollup, "Pss");
    assert.equal(member.rssKiB, rollupRss);
    assert.equal(member.pssKiB, rollupPss);
    assert.ok(rollupPss <= rollupRss);
    kib(member.status, "VmRSS");
    rss += rollupRss;
    pss += rollupPss;
    assert.ok(Array.isArray(member.cmdlineBytes));
    const argv = Buffer.from(member.cmdlineBytes as number[])
      .toString("utf8")
      .split("\0");
    if (index === 0) {
      rootPid = before.pid;
      assert.equal(rootPid, expectedPid);
      assert.equal(member.cwd, sourceRoot);
      assert.equal(member.executable, executable);
      assert.equal(member.executableSHA256, testHash);
      assert.ok(argv.includes(SELECTOR));
    } else {
      assert.equal(before.ppid, rootPid);
      assert.equal(member.executable, native);
      assert.equal(member.executableSHA256, IMAGE);
      assert.equal(member.cwd, input);
      assert.ok(argv.includes("--lsp"));
    }
  }
  assert.equal(row.treeRssKiB, rss);
  assert.equal(row.treePssKiB, pss);
  assert.ok(rss < MEMORY_LIMIT_KIB, "prospective 2GiB checkpoint ceiling");
}
export function controls(): void {
  let passed = 0;
  const yes = (fn: () => void): void => {
      fn();
      passed++;
    },
    no = (fn: () => void): void => {
      assert.throws(fn);
      passed++;
    };
  const fixture =
    "42 (test (nested) name) S 10 " +
    Array.from({ length: 20 }, (_, i) => (i === 17 ? "1234" : "0")).join(" ");
  yes(() => assert.deepEqual(stat(fixture), { pid: 42, ppid: 10, birth: 1234, state: "S" }));
  for (const value of [
    null,
    {},
    "",
    "42 (test) S 1",
    fixture.replace(") S ", ") Z "),
    fixture.replace("1234", "NaN"),
    fixture.replace("42 (", "0 ("),
    fixture.replace(") S 10", ") S -1"),
  ])
    no(() => stat(value));
  yes(() => assert.equal(kib("Rss:  123 kB\nPss: 42 kB\n", "Pss"), 42));
  for (const value of [
    null,
    "Pss: 4 B",
    "Pss: -4 kB",
    "Pss: NaN kB",
    "Pss: 4 kB\nPss: 4 kB",
    "Rss: 4 kB",
  ])
    no(() => kib(value, "Pss"));
  yes(() => assert.deepEqual(ORDER, ["baseline", "candidate", "candidate", "baseline"]));
  yes(() =>
    assert.equal(SELECTOR.endsWith("::native_two_config_owner_resource_discriminator"), true),
  );
  for (const value of [null, [], 42, "x"]) no(() => object(value));
  const proc = (pid: number, ppid: number, birth: number): string =>
    `${pid} (test) S ${ppid} ` +
    Array.from({ length: 20 }, (_, i) => (i === 17 ? String(birth) : "0")).join(" ");
  const member = (
    pid: number,
    ppid: number,
    exe: string,
    image: string,
    cwd: string,
    argv: string[],
  ): Record<string, unknown> => ({
    pid,
    birth: 1234,
    executable: exe,
    executableSHA256: image,
    cwd,
    cmdlineBytes: [...Buffer.from(argv.join("\0") + "\0")],
    statBefore: proc(pid, ppid, 1234),
    statAfter: proc(pid, ppid, 1234),
    status: "VmRSS: 20 kB\n",
    smapsRollup: "Rss: 20 kB\nPss: 10 kB\n",
    rssKiB: 20,
    pssKiB: 10,
    collectionStartUnixUs: 2,
    collectionEndUnixUs: 3,
  });
  const valid: Record<string, unknown> = {
    collectionStartUnixUs: 1,
    collectionEndUnixUs: 4,
    processCount: 2,
    rows: [
      member(42, 1, "/test", "testhash", "/source", ["/test", SELECTOR]),
      member(43, 42, "/native", IMAGE, "/input", ["/native", "--lsp", "--stdio"]),
    ],
    treeRssKiB: 40,
    treePssKiB: 20,
  };
  const verify = (row: Record<string, unknown>): void =>
    memory(row, 1, "/test", "testhash", "/native", "/input", 42, "/source");
  yes(() => verify(valid));
  const change = (
    fn: (row: Record<string, unknown>, members: Record<string, unknown>[]) => void,
  ): void => {
    const copy = structuredClone(valid);
    fn(copy, copy.rows as Record<string, unknown>[]);
    verify(copy);
  };
  for (const mutate of [
    (row: Record<string, unknown>) => {
      row.processCount = 3;
    },
    (row: Record<string, unknown>) => {
      row.treeRssKiB = 41;
    },
    (row: Record<string, unknown>) => {
      row.treePssKiB = 19;
    },
    (row: Record<string, unknown>) => {
      row.collectionEndUnixUs = 0;
    },
  ])
    no(() => change(mutate));
  for (const [key, value] of [
    ["pid", 42],
    ["birth", 999],
    ["executable", "/foreign"],
    ["executableSHA256", "foreign"],
    ["cwd", "/foreign"],
    ["rssKiB", 21],
    ["pssKiB", 21],
    ["statAfter", proc(43, 42, 999)],
    ["statBefore", proc(43, 99, 1234)],
    ["collectionStartUnixUs", 0],
    ["collectionEndUnixUs", 99],
    ["cmdlineBytes", [...Buffer.from("/native\0--api\0")]],
  ])
    no(() =>
      change((_row, members) => {
        members[1]![String(key)] = value;
      }),
    );
  console.log(JSON.stringify({ passed, native: false, network: false }));
}
