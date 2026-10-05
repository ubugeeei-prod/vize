import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const validator = String.raw`
import hashlib,json,pathlib,stat,sys,tarfile,zipfile
kind,archive,destination=sys.argv[1:]
root=pathlib.Path(destination);root.mkdir(parents=True,exist_ok=False)
rows=[]
if kind=='zip':
 with zipfile.ZipFile(archive) as z:
  entries=z.infolist();assert len(entries)<=10000
  assert sum(e.file_size for e in entries)<=512*1024*1024
  seen=set()
  for e in entries:
   p=pathlib.PurePosixPath(e.filename)
   assert not p.is_absolute() and '..' not in p.parts and '\\' not in e.filename
   assert e.filename not in seen;seen.add(e.filename)
   assert not e.flag_bits&1
   assert stat.S_IFMT(e.external_attr>>16) in (0,stat.S_IFREG,stat.S_IFDIR)
   h=hashlib.sha256();n=0
   target=root/p
   assert p.parts and p.parts[0]=='warm-pair'
   if e.is_dir():target.mkdir(parents=True,exist_ok=True);continue
   target.parent.mkdir(parents=True,exist_ok=True)
   with z.open(e) as source,target.open('xb') as out:
    while data:=source.read(1024*1024):h.update(data);n+=len(data);out.write(data)
   assert n==e.file_size
   rows.append({'path':e.filename,'bytes':n,'sha256':h.hexdigest(),'crc32':e.CRC})
else:
 assert kind=='tar'
 with tarfile.open(archive,'r:gz') as t:
  entries=t.getmembers();assert len(entries)==1
  e=entries[0];assert e.name=='vize' and e.isfile() and 0<e.size<256*1024*1024
  assert e.mode&0o111
  source=t.extractfile(e);assert source is not None
  h=hashlib.sha256();n=0;target=root/'vize'
  with target.open('xb') as out:
   while data:=source.read(1024*1024):h.update(data);n+=len(data);out.write(data)
  assert n==e.size;target.chmod(e.mode&0o777)
  rows.append({'path':e.name,'bytes':n,'mode':e.mode,'sha256':h.hexdigest()})
(root/'archive-audit.json').write_text(json.dumps({'kind':kind,'members':rows},indent=2)+'\n')
`;

/** Validate complete official archives before opening any captured authority. */
export function unpackPublicArchive(kind: "zip" | "tar", archive: string, destination: string) {
  const result = spawnSync("python3", ["-c", validator, kind, archive, destination], {
    maxBuffer: 64 * 1024 * 1024,
  });
  fs.writeFileSync(
    `${destination}.process.json`,
    `${JSON.stringify({ status: result.status, signal: result.signal, error: result.error && String(result.error), stdoutBase64: result.stdout.toString("base64"), stderrBase64: result.stderr.toString("base64") }, null, 2)}\n`,
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr.toString());
  return JSON.parse(fs.readFileSync(path.join(destination, "archive-audit.json"), "utf8"));
}
