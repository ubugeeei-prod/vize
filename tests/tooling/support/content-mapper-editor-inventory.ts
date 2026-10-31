import assert from "node:assert/strict";

export function names(text: string, producer: string = "libtest") {
  const result = [];
  let timingRows = 0;
  for (const line of text.split("\n").filter(Boolean)) {
    const match = /^(.*): test$/u.exec(line);
    if (match) {
      assert.equal(timingRows, 0, "Rustdoc timing must follow all case rows");
      result.push(match[1]);
    } else {
      assert.equal(producer, "rustdoc", `unexpected complete terse inventory row: ${line}`);
      assert.match(
        line,
        /^all doctests ran in \d+\.\d{2}s; merged doctests compilation took \d+\.\d{2}s$/u,
      );
      assert.equal(++timingRows, 1, "one exact Rustdoc timing record");
    }
  }
  // Terse inventories contain case rows only. Each full execution below
  // independently proves the exact same names and its complete summary count.
  assert.equal(new Set(result).size, result.length, "inventory names must be unique per target");
  return result.sort();
}
export function executions(
  text: string,
  inventory: readonly string[],
  ignored: readonly string[],
  selected: readonly string[],
  producer: string = "libtest",
) {
  const events = [...text.matchAll(/^test (.+) \.\.\. (ok|FAILED|ignored(?:,.*)?)$/gmu)].map(
    (match) => {
      const rawName = match[1];
      const shouldPanic = rawName.endsWith(" - should panic");
      const withoutPanic = shouldPanic ? rawName.slice(0, -" - should panic".length) : rawName;
      const compile = producer === "rustdoc" && withoutPanic.endsWith(" - compile");
      const state = match[2].startsWith("ignored") ? "ignored" : match[2];
      return {
        name: compile ? withoutPanic.slice(0, -" - compile".length) : withoutPanic,
        rawName,
        expectation: shouldPanic ? "panic" : "normal",
        mode: state === "ignored" ? "ignored" : compile ? "compile" : "run",
        state,
      };
    },
  );
  assert.deepEqual(
    events.map((event) => event.name).sort(),
    [...selected].sort(),
    "all selected cases must produce exactly one complete result",
  );
  assert.deepEqual(
    events
      .filter((event) => event.state === "ignored")
      .map((event) => event.name)
      .sort(),
    ignored.filter((name) => selected.includes(name)),
    "declared ignored-case policy must remain unchanged",
  );
  assert.equal(
    events.some((event) => event.state === "FAILED"),
    false,
  );
  const summaries = [
    ...text.matchAll(
      /^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;/gmu,
    ),
  ];
  if (producer === "libtest") assert.equal(summaries.length, 1, "one complete libtest summary");
  else assert.ok(summaries.length > 0, "every Rustdoc harness must complete");
  const actual = summaries.reduce(
    (total, summary) => total.map((value, index) => value + Number(summary[index + 1])),
    [0, 0, 0, 0, 0],
  );
  const ignoredCount = ignored.filter((name) => selected.includes(name)).length;
  assert.deepEqual(
    actual,
    [selected.length - ignoredCount, 0, ignoredCount, 0, inventory.length - selected.length],
    "complete inventory and summary counts must agree",
  );
  return events;
}
export function reject(label: string, operation: () => unknown) {
  let rejected = false;
  try {
    operation();
  } catch {
    rejected = true;
  }
  assert.equal(rejected, true, `${label}: evidence must fail closed`);
}
