import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import {
  archiveMember,
  GitHub,
  sha256,
  stableJson,
} from "../../tools/support/compat/github/docs-deployment-api.ts";
import { artifact, manifest, run } from "./support/docs-deployment.ts";

function archive(mode: string, path = "_og/manifest.json", content = JSON.stringify(manifest)) {
  const script = `import io,json,sys,tarfile,zipfile
mode,path,content=json.loads(sys.stdin.read())
buffer=io.BytesIO()
with zipfile.ZipFile(buffer,'w') as archive:
 if mode.startswith('pages'):
  tar=io.BytesIO()
  with tarfile.open(fileobj=tar,mode='w') as site:
   member=tarfile.TarInfo('./'+path)
   member.size=len(content.encode())
   if mode=='pages-link': member.type=tarfile.SYMTYPE;member.linkname='elsewhere'
   site.addfile(member,io.BytesIO(content.encode()))
  archive.writestr('artifact.tar',tar.getvalue())
 else:
  archive.writestr(path,content)
  if mode=='duplicate': archive.writestr(path,content)
sys.stdout.buffer.write(buffer.getvalue())`;
  const result = spawnSync("python3", ["-c", script], {
    input: JSON.stringify([mode, path, content]),
    maxBuffer: 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr.toString());
  return result.stdout;
}

test("native archive reads bind whole primary digests and preserve the original manifest bytes", async () => {
  const native = JSON.stringify(manifest, null, 2) + "\n";
  const docs = archive("docs", "_og/manifest.json", native);
  const pages = archive("pages", "_og/manifest.json", native);
  assert.equal(archiveMember(docs, false).toString(), native);
  assert.equal(archiveMember(pages, true).toString(), native);
  const api = new GitHub("ubugeeei-prod/vize", "fixture-credential");
  api.binary = async () => docs;
  const metadata = { ...artifact(run(), "docs"), digest: "sha256:" + sha256(docs) };
  assert.deepEqual(await api.manifest(metadata), {
    sourceSha: manifest.sourceSha,
    assetFingerprint: manifest.assetFingerprint,
    sha256: sha256(native),
  });
  await assert.rejects(
    () => api.manifest({ ...metadata, digest: "sha256:" + "f".repeat(64) }),
    /archive digest/,
  );
  for (const bytes of [
    archive("duplicate"),
    archive("docs", "../_og/manifest.json"),
    archive("docs", "other.json"),
  ]) {
    assert.throws(() => archiveMember(bytes, false));
  }
  assert.throws(() => archiveMember(archive("pages-link"), true), /ordinary native archive member/);
  assert.equal(
    stableJson({ source: { z: 1, a: 2 }, pages: [3, 4] }),
    stableJson({ pages: [3, 4], source: { a: 2, z: 1 } }),
  );
});

test("primary workflow searches refuse overflow, truncation and changing totals", async () => {
  const api = new GitHub("ubugeeei-prod/vize", "fixture-credential");
  const path = "/actions/workflows/deploy-docs.yml/runs?branch=main&created=>=1970-01-01";
  let packets: unknown[] = [];
  const requests: string[] = [];
  api.json = async <T>(request: string): Promise<T> => {
    requests.push(request);
    assert(packets.length, "Every primary page has an explicit fixture");
    return packets.shift() as T;
  };
  for (const total of [1001, undefined, -1, 1.5]) {
    packets = [{ total_count: total, workflow_runs: [{ id: 1 }] }];
    await assert.rejects(() => api.list(path, "workflow_runs"), /primary total_count/);
  }
  packets = [{ total_count: 2, workflow_runs: [{ id: 1 }] }];
  await assert.rejects(() => api.list(path, "workflow_runs"), /Complete primary workflow history/);
  const first = Array.from({ length: 100 }, (_, index) => ({ id: index + 1 }));
  packets = [
    { total_count: 101, workflow_runs: first },
    { total_count: 102, workflow_runs: [{ id: 101 }] },
  ];
  await assert.rejects(() => api.list(path, "workflow_runs"), /Stable primary workflow history/);
  packets = [
    { total_count: 101, workflow_runs: first },
    { total_count: 101, workflow_runs: [{ id: 101 }] },
  ];
  assert.deepEqual(await api.list(path, "workflow_runs"), [...first, { id: 101 }]);
  assert(requests.at(-1)?.endsWith("&per_page=100&page=2"));
  packets = [{ total_count: 0, workflow_runs: [] }];
  assert.deepEqual(await api.list(path, "workflow_runs"), []);
  // Artifact/job pagination has no workflow-search cap or total_count requirement.
  packets = [{ artifacts: [{ id: 1 }] }];
  assert.deepEqual(await api.list("/actions/runs/1/artifacts", "artifacts"), [{ id: 1 }]);
});
