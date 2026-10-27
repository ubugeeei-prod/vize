/** Untimed direct backend observer; no shell or user-source evaluation. */
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export function createColdBackendObserver(backend: string, out: string) {
  const calls = join(out, "backend");
  mkdirSync(calls, { recursive: true });
  const wrapper = join(out, "backend-observer.cjs");
  const source = `#!${process.execPath}
const fs = require('node:fs'), path = require('node:path'), cp = require('node:child_process'), crypto = require('node:crypto');
const backend = ${JSON.stringify(backend)}, root = ${JSON.stringify(calls)};
const startedMonotonicNs = process.hrtime.bigint().toString();
const args = process.argv.slice(2), directory = fs.mkdtempSync(path.join(root, 'call-'));
const value = (name) => { const i=args.indexOf(name); return i<0?null:args[i+1]??null; };
function physical(p) {
  const suffix=[]; let cursor=path.resolve(p);
  while(!fs.existsSync(cursor)) { const parent=path.dirname(cursor); if(parent===cursor)break; suffix.unshift(path.basename(cursor)); cursor=parent; }
  try { return path.join(fs.realpathSync(cursor), ...suffix); } catch { return null; }
}
function fileHash(p) {
  const fd=fs.openSync(p,'r'), hash=crypto.createHash('sha256'), buffer=Buffer.allocUnsafe(64*1024);
  try { let count; while((count=fs.readSync(fd,buffer,0,buffer.length,null))>0)hash.update(buffer.subarray(0,count)); return hash.digest('hex'); }
  finally { fs.closeSync(fd); }
}
function snapshot(p, hashContents=false) {
  if(!p)return null;
  try { const s=fs.statSync(p); return {path:p,physicalPath:physical(p),exists:true,bytes:s.size,modifiedMs:s.mtimeMs,sha256:hashContents&&s.isFile()?fileHash(p):null}; }
  catch(e) { return {path:p,physicalPath:physical(p),exists:e.code==='ENOENT'||e.code==='ENOTDIR'?false:null,error:e.code}; }
}
const project=value('--project'), buildInfo=value('--tsBuildInfoFile');
let config=null, configSha256=null;
if(project) { const bytes=fs.readFileSync(project); fs.writeFileSync(path.join(directory,'generated-tsconfig.json'),bytes); config=JSON.parse(bytes); configSha256=crypto.createHash('sha256').update(bytes).digest('hex'); }
const before=snapshot(buildInfo), incrementalIndex=args.indexOf('--incremental');
const identity={startedMonotonicNs,observerPid:process.pid,argv:args,cwd:process.cwd(),physicalCwd:fs.realpathSync('.'),project,configSha256,configuredIncremental:config?.compilerOptions?.incremental??null,effectiveIncremental:incrementalIndex>=0?args[incrementalIndex+1]!=='false':config?.compilerOptions?.incremental??false,before,environment:{XDG_CACHE_HOME:process.env.XDG_CACHE_HOME??null,TMPDIR:process.env.TMPDIR??null}};
fs.writeFileSync(path.join(directory,'before.json'),JSON.stringify(identity,null,2)+'\\n');
const result=cp.spawnSync(backend,args,{encoding:'utf8',env:process.env,maxBuffer:64*1024*1024,timeout:600000});
fs.writeFileSync(path.join(directory,'stdout.txt'),result.stdout??''); fs.writeFileSync(path.join(directory,'stderr.txt'),result.stderr??'');
fs.writeFileSync(path.join(directory,'after.json'),JSON.stringify({backendPid:result.pid??null,status:result.status,signal:result.signal,error:result.error?.message??null,buildInfo:snapshot(buildInfo,true)},null,2)+'\\n');
process.stdout.write(result.stdout??''); process.stderr.write(result.stderr??'');
if(result.error)process.exit(125); if(result.signal)process.kill(process.pid,result.signal); else process.exit(result.status??125);
`;
  writeFileSync(wrapper, source, { flag: "wx", mode: 0o755 });
  return { wrapper, calls };
}
