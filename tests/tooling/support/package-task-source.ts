import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/** Read the generated command data without running code from a fixture checkout. */
export function packageCommands(
  root: string,
  directory: string,
  script: string | undefined,
  name: string,
): string[] {
  assert.equal(script, "node ./scripts/package-tasks.cjs " + name);
  const source = readFileSync(path.join(root, directory, "scripts/package-tasks.cjs"), "utf8");
  const start = "\nconst tasks = ",
    end = ";\nexports.tasks = tasks;\n";
  const offset = source.lastIndexOf(start);
  assert.ok(
    offset >= 0 &&
      source.endsWith(
        "if (require.main === module) process.exitCode = exports.runPackageTask(tasks, process.argv.slice(2));\n",
      ),
  );
  const finish = source.indexOf(end, offset);
  assert.ok(finish > offset);
  const tasks = JSON.parse(source.slice(offset + start.length, finish)) as Record<
    string,
    {
      program: string;
      arguments?: string[];
      globs?: string[];
      defaults?: Record<string, string>;
      environment?: Record<string, string>;
      directory?: string;
      absolutePaths?: string[];
    }[]
  >;
  assert.ok(Array.isArray(tasks[name]));
  return tasks[name]!.map((step) => {
    assert.deepEqual(
      Object.keys(step).filter(
        (key) =>
          ![
            "program",
            "arguments",
            "globs",
            "directory",
            "environment",
            "defaults",
            "absolutePaths",
          ].includes(key),
      ),
      [],
    );
    const env = [
      ...Object.entries(step.environment ?? {}).map(([name, value]) => name + "=" + value),
      ...Object.entries(step.defaults ?? {}).map(
        ([name, value]) => name + "=${" + name + ":-" + value + "}",
      ),
    ];
    const args = (step.arguments ?? []).map((arg) => {
      if (step.absolutePaths?.includes(arg)) return path.resolve(root, directory, arg);
      if ((arg.includes("*") && !step.globs?.includes(arg)) || /\s/u.test(arg))
        return "'" + arg.replaceAll("'", "'\\''") + "'";
      return arg;
    });
    return [
      ...(step.directory ? ["cd", step.directory, "&&"] : []),
      ...env,
      step.program,
      ...args,
    ].join(" ");
  });
}

export function packageScripts(
  directory: string,
  scripts: Record<string, string> | undefined,
): Record<string, string> {
  const workspaceRoot = fileURLToPath(new URL("../../../", import.meta.url));
  return Object.fromEntries(
    Object.entries(scripts ?? {}).map(([name, command]) => [
      name,
      command.startsWith("node ./scripts/package-tasks.cjs ")
        ? packageCommands(workspaceRoot, directory, command, name).join(" && ")
        : command,
    ]),
  );
}
