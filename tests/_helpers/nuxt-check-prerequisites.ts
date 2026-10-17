import assert from "node:assert/strict";
import * as path from "node:path";

import { npmxGeneratorTaskArgs } from "./app-fixture-runtime.ts";
import { walkFiles } from "./nuxt-check-source.ts";

export function authoredNuxtPrerequisiteTasks(
  name: string,
  postinstall: unknown,
): readonly ("generate:lexicons" | "generate:sprite")[] {
  if (name !== "npmx.dev") return [];
  assert.equal(typeof postinstall, "string", "npmX must retain its authored postinstall");
  const tasks = ["generate:lexicons", "generate:sprite"] as const;
  for (const task of tasks) {
    assert.ok(
      (postinstall as string).includes(`vp run ${task}`),
      `npmX no longer authors the ${task} prerequisite`,
    );
  }
  return tasks;
}

/** These are the pinned npmX postinstall's generators, without its source rewrite. */
export function prepareNuxtPrerequisites(
  name: string,
  postinstall: unknown,
  command: (args: string[], timeout: number) => void,
): void {
  for (const task of authoredNuxtPrerequisiteTasks(name, postinstall))
    command(npmxGeneratorTaskArgs(task), 120_000);
}

export function retainNuxtPrerequisites(
  name: string,
  cwd: string,
  retain: (file: string, role: string) => void,
): void {
  if (name !== "npmx.dev") return;
  walkFiles(path.join(cwd, "shared/types/lexicons"), (file) =>
    retain(file, "generated-provider-prerequisite"),
  );
  for (const publicDir of ["public", "public-dev", "public-staging"]) {
    retain(path.join(cwd, publicDir, "file-tree-sprite.svg"), "generated-provider-prerequisite");
  }
}
