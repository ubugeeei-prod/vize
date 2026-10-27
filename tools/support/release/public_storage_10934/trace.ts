import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { asArray, asString, inside, parse, save, sha256 } from "./io.ts";

export function attestGoExecutions(evidence: string, backend: string, virtualRoot: string) {
  const files = fs.readdirSync(evidence).filter((name) => /^exec\.[0-9]+$/.test(name));
  assert.ok(files.length > 0, "strace must retain actual process exec observations");
  const calls: Array<{
    pid: number; trace: string; executable: string; args: string[]; cwd: string;
    requestedConfig: string; postRunConfigSha256: string; postRunConfigBytes: number;
  }> = [];
  for (const name of files) {
    const file = path.join(evidence, name);
    const text = new TextDecoder("utf-8", { fatal: true }).decode(fs.readFileSync(file));
    let cwd: string | undefined;
    for (const line of text.split("\n")) {
      const directory = line.match(/^chdir\(("(?:\\.|[^"\\])*")\)\s+= 0$/);
      if (directory) {
        assert.ok(directory[1]);
        cwd = asString(parse(Buffer.from(directory[1])));
      }
      const executed = line.match(/^execve\(("(?:\\.|[^"\\])*"), (\[.*\]), .*\)\s+= 0$/);
      if (!executed) continue;
      assert.ok(executed[1] && executed[2]);
      const executable = asString(parse(Buffer.from(executed[1])));
      if (executable !== backend) continue;
      const args = asArray(parse(Buffer.from(executed[2]))).map((value) => asString(value));
      const index = args.indexOf("--project");
      if (index === -1) continue;
      assert.equal(cwd, virtualRoot, "actual native checker cwd must be the owned virtual root");
      const requested = args[index + 1];
      assert.ok(requested && path.isAbsolute(requested));
      const config = inside(virtualRoot, requested);
      const bytes = fs.readFileSync(config);
      fs.writeFileSync(path.join(evidence, "requested-config.post-" + sha256(bytes) + ".json"), bytes);
      calls.push({
        pid: Number(name.slice("exec.".length)), trace: name, executable, args,
        cwd: virtualRoot, requestedConfig: config,
        postRunConfigSha256: sha256(bytes), postRunConfigBytes: bytes.length,
      });
    }
  }
  assert.equal(calls.length, 3, "all three fresh check processes must invoke the real public native backend");
  assert.equal(new Set(calls.map((item) => item.pid)).size, 3);
  assert.equal(new Set(calls.map((item) => item.requestedConfig)).size, 1);
  save(path.join(evidence, "actual-go-executions.json"), {
    observer: "strace -ff; successful execve and chdir from direct descendants",
    calls,
    configObservation: "post-three-process bytes; pre-invocation bytes remain unobserved",
    rustSemanticRootAuthority: "not attested by this external observer",
    timingQualification: false,
  });
  return calls;
}
