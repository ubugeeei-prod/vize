import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
export type Identity = { path: string; bytes: number; sha256: string };
export const hash = (bytes: string | Buffer): string =>
  createHash("sha256").update(bytes).digest("hex");
export function save(file: string, value: unknown): void {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}
export function identify(file: string): Identity {
  const actual = fs.realpathSync(file),
    bytes = fs.readFileSync(actual);
  return { path: actual, bytes: bytes.length, sha256: hash(bytes) };
}
export function run(
  output: string,
  name: string,
  executable: string,
  argv: string[],
  cwd: string,
  env: NodeJS.ProcessEnv = process.env,
): { status: number | null; stdout: Buffer; stderr: Buffer } {
  const result = spawnSync(executable, argv, {
    cwd,
    env,
    timeout: 90_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const prefix = path.join(output, name),
    stdout = result.stdout ?? Buffer.alloc(0),
    stderr = result.stderr ?? Buffer.alloc(0);
  fs.mkdirSync(path.dirname(prefix), { recursive: true });
  fs.writeFileSync(prefix + ".stdout.bin", stdout);
  fs.writeFileSync(prefix + ".stderr.bin", stderr);
  save(prefix + ".process.json", {
    executable: identify(executable),
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdoutSha256: hash(stdout),
    stderrSha256: hash(stderr),
    environment: Object.fromEntries(
      ["CORSA_PATH", "PATH", "GOMAXPROCS", "RAYON_NUM_THREADS", "NODE_OPTIONS", "NODE_PATH"].map(
        (key) => [key, env[key] ?? null],
      ),
    ),
  });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null);
  return { status: result.status, stdout, stderr };
}
export const wrapperSource = [
  "#!/usr/bin/env node",
  "import fs from 'node:fs'; import path from 'node:path'; import {spawnSync} from 'node:child_process';",
  "const root=process.env.VIZE_DIRECTIVE_NATIVE_CAPTURE; fs.mkdirSync(root,{recursive:true});",
  "const out=fs.mkdtempSync(path.join(root,'call-')),argv=process.argv.slice(2);",
  "fs.writeFileSync(path.join(out,'request.json'),JSON.stringify({executable:process.env.VIZE_DIRECTIVE_NATIVE,argv,cwd:process.cwd()}));",
  "const pi=argv.indexOf('--project'); if(pi>=0){const cfg=path.resolve(process.cwd(),argv[pi+1]);fs.copyFileSync(cfg,path.join(out,'tsconfig.json'));}",
  "const r=spawnSync(process.env.VIZE_DIRECTIVE_NATIVE,argv,{env:process.env,cwd:process.cwd(),maxBuffer:64*1024*1024,timeout:85000});",
  "fs.writeFileSync(path.join(out,'stdout.bin'),r.stdout??Buffer.alloc(0));fs.writeFileSync(path.join(out,'stderr.bin'),r.stderr??Buffer.alloc(0));",
  "fs.writeFileSync(path.join(out,'response.json'),JSON.stringify({status:r.status,signal:r.signal,error:r.error?.message??null}));",
  "if(r.stdout?.length)await new Promise(resolve=>process.stdout.write(r.stdout,resolve));",
  "if(r.stderr?.length)await new Promise(resolve=>process.stderr.write(r.stderr,resolve));",
  "if(r.signal)process.kill(process.pid,r.signal);else process.exitCode=r.status??1;",
  "",
].join("\n");
