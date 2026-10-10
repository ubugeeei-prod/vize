import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { Artifact, ManifestIdentity } from "./docs-deployment-policy.ts";

export const sha256 = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");
export function stableJson(value: unknown): string {
  function ordered(item: unknown): unknown {
    if (Array.isArray(item)) return item.map(ordered);
    if (item !== null && typeof item === "object")
      return Object.fromEntries(
        Object.entries(item)
          .sort(([a], [b]) => a.localeCompare(b))
          .map(([key, child]) => [key, ordered(child)]),
      );
    return item;
  }
  return JSON.stringify(ordered(value));
}
export type Deployment = {
  id: number;
  created_at: string;
  sha: string;
  task: string;
  environment: string;
  creator: { login: string; type: string };
  payload: unknown;
};

// Read a single member; neither archive can write paths, symlinks or executable code.
export function archiveMember(bytes: Uint8Array, pages: boolean, member = "_og/manifest.json") {
  const directory = mkdtempSync(join(tmpdir(), "vize-pages-custody-"));
  const file = join(directory, "artifact.zip");
  try {
    writeFileSync(file, bytes);
    const script = `import io,sys,tarfile,zipfile
with zipfile.ZipFile(sys.argv[1]) as archive:
 if sys.argv[2] == 'pages':
  with tarfile.open(fileobj=io.BytesIO(archive.read('artifact.tar'))) as site:
   members=[m for m in site.getmembers() if m.name in (sys.argv[3], './'+sys.argv[3])]
   assert len(members)==1 and members[0].isfile() and members[0].size<=16777216
   sys.stdout.buffer.write(site.extractfile(members[0]).read())
 else:
  members=[m for m in archive.infolist() if m.filename in (sys.argv[3], './'+sys.argv[3])]
  assert len(members)==1 and not members[0].is_dir() and members[0].file_size<=16777216
  sys.stdout.buffer.write(archive.read(members[0]))`;
    const result = spawnSync("python3", ["-c", script, file, pages ? "pages" : "docs", member], {
      maxBuffer: 16 * 1024 * 1024,
    });
    assert.equal(
      result.status,
      0,
      "One bounded ordinary native archive member: " + result.stderr.toString(),
    );
    return result.stdout;
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

export class GitHub {
  repository: string;
  token: string;
  constructor(repository: string, token: string) {
    assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/);
    assert(token, "GitHub token is required");
    this.repository = repository;
    this.token = token;
  }
  async response(path: string, body?: unknown) {
    assert(path === "" || (path.startsWith("/") && !path.startsWith("//")));
    return await fetch("https://api.github.com/repos/" + this.repository + path, {
      method: body === undefined ? "GET" : "POST",
      headers: {
        authorization: "Bearer " + this.token,
        accept: "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "content-type": "application/json",
      },
      body: body === undefined ? undefined : JSON.stringify(body),
      redirect: "manual",
    });
  }
  async json<T>(path: string, body?: unknown): Promise<T> {
    const response = await this.response(path, body);
    assert(response.ok, "GitHub metadata request failed: " + response.status + " " + path);
    return (await response.json()) as T;
  }
  async binary(path: string) {
    let response = await this.response(path);
    if (response.status === 302) {
      const url = new URL(response.headers.get("location") ?? "");
      assert.equal(url.protocol, "https:");
      // The signed storage URL receives no GitHub credential.
      response = await fetch(url, { redirect: "error" });
    }
    assert(response.ok, "GitHub archive request failed: " + response.status + " " + path);
    return new Uint8Array(await response.arrayBuffer());
  }
  async list<T>(path: string, key?: string) {
    const values: T[] = [];
    let total: number | undefined;
    for (let page = 1; ; page++) {
      const result = await this.json<T[] | Record<string, unknown>>(
        path + (path.includes("?") ? "&" : "?") + "per_page=100&page=" + page,
      );
      const packet = result as Record<string, unknown>;
      if (key === "workflow_runs") {
        const count = packet.total_count;
        assert(
          typeof count === "number" && Number.isSafeInteger(count) && count >= 0 && count <= 1000,
          "Complete workflow history requires a primary total_count within the 1000-run search limit",
        );
        if (total !== undefined)
          assert.equal(count, total, "Stable primary workflow history count");
        total = count;
      }
      const batch = key ? packet[key] : result;
      assert(Array.isArray(batch), "Complete primary metadata page");
      values.push(...batch);
      if (batch.length < 100) {
        if (total !== undefined)
          assert.equal(values.length, total, "Complete primary workflow history");
        return values;
      }
    }
  }
  async manifest(artifact: Artifact, pages = false): Promise<ManifestIdentity> {
    const bytes = await this.binary("/actions/artifacts/" + artifact.id + "/zip");
    assert.equal(
      "sha256:" + sha256(bytes),
      artifact.digest,
      "Whole primary artifact archive digest",
    );
    const native = archiveMember(bytes, pages);
    const manifest = JSON.parse(native.toString()) as {
      sourceSha: string;
      assetFingerprint: string;
    };
    return {
      sourceSha: manifest.sourceSha,
      assetFingerprint: manifest.assetFingerprint,
      sha256: sha256(native),
    };
  }
  async jobLog(jobId: number) {
    return Buffer.from(await this.binary("/actions/jobs/" + jobId + "/logs")).toString();
  }
  async workflowHash(source: string) {
    const file = await this.json<{ encoding: string; content: string }>(
      "/contents/.github/workflows/deploy-docs.yml?ref=" + source,
    );
    assert.equal(file.encoding, "base64");
    return sha256(Buffer.from(file.content, "base64"));
  }
}
