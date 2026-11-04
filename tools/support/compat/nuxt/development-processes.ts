import assert from "node:assert/strict";

export function verifyDevelopmentProcesses(
  launcher: number,
  observed: readonly unknown[],
  snapshot: string,
): void {
  const parents = new Map<number, number>();
  for (const line of snapshot.trim().split("\n")) {
    const fields = /^\s*(\d+)\s+(\d+)\s*$/.exec(line);
    assert.ok(fields, "invalid process ancestry observation");
    const pid = Number(fields[1]);
    const parent = Number(fields[2]);
    assert.ok(Number.isSafeInteger(pid) && Number.isSafeInteger(parent));
    assert.ok(!parents.has(pid), "duplicate process ancestry observation");
    parents.set(pid, parent);
  }
  assert.ok(parents.has(launcher), "the Nuxt launcher exited before custody verification");
  for (const pid of observed) {
    assert.ok(typeof pid === "number" && Number.isSafeInteger(pid) && pid > 0);
    let current = pid;
    const visited = new Set<number>();
    while (current !== launcher) {
      assert.ok(!visited.has(current), "cyclic process ancestry cannot qualify a compiler");
      visited.add(current);
      const parent = parents.get(current);
      assert.ok(parent !== undefined && parent > 0, "compiler is outside the Nuxt process tree");
      current = parent;
    }
  }
}
