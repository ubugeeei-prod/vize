import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { asRecord } from "../../tools/support/json-value.ts";
import {
  profileFor,
  sha256,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import { recorder } from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import {
  e649MetadataAccept,
  e649MetadataUrl,
  readE649Metadata,
  selectE649Metadata,
  usesE649Packument,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/e649-metadata.ts";

// Whole genuine selected member of the retained 773435-byte install packument.
const memberBase64 =
  "eyJuYW1lIjoidml6ZSIsInZlcnNpb24iOiIwLjY0OS4wIiwiZGVwZW5kZW5jaWVzIjp7Im94Yy10cmFuc2Zvcm0iOiIwLjE0NC4wIiwiQHZpemVqcy9uYXRpdmUiOiIwLjY0OS4wIn0sIm9wdGlvbmFsRGVwZW5kZW5jaWVzIjp7IkB0eXBlc2NyaXB0L3R5cGVzY3JpcHQtbGludXgteDY0IjoiNy4wLjIiLCJAdHlwZXNjcmlwdC90eXBlc2NyaXB0LXdpbjMyLXg2NCI6IjcuMC4yIiwiQHR5cGVzY3JpcHQvdHlwZXNjcmlwdC1kYXJ3aW4teDY0IjoiNy4wLjIiLCJAdHlwZXNjcmlwdC90eXBlc2NyaXB0LWxpbnV4LWFybTY0IjoiNy4wLjIiLCJAdHlwZXNjcmlwdC90eXBlc2NyaXB0LXdpbjMyLWFybTY0IjoiNy4wLjIiLCJAdHlwZXNjcmlwdC90eXBlc2NyaXB0LWRhcndpbi1hcm02NCI6IjcuMC4yIn0sImRldkRlcGVuZGVuY2llcyI6eyJ2dWUiOiIzLjUuMzUiLCJ2aXRlIjoibnBtOkB2b2lkemVyby1kZXYvdml0ZS1wbHVzLWNvcmVAMC4xLjI0IiwidHNkb3duIjoiMC4yMi4xNCIsInZpdGUtcGx1cyI6IjAuMS4yNCIsInR5cGVzY3JpcHQiOiI2LjAuMyIsIkB0c2Rvd24vY3NzIjoiMC4yMi4xNCIsIkB0eXBlcy9ub2RlIjoiMjUuOS4yIiwiQHBrbC1jb21tdW5pdHkvcGtsIjoiMC4zMC4yIn0sInBlZXJEZXBlbmRlbmNpZXMiOnsidnVlIjoiPj0yLjYuMCB8fCA+PTMuNi4wLTAiLCJAcGtsLWNvbW11bml0eS9wa2wiOiIwLjMwLjIifSwiYmluIjp7InZ6IjoiYmluL3ZpemUiLCJ2aXplIjoiYmluL3ZpemUifSwiZGlzdCI6eyJzaGFzdW0iOiJiOGVlYWEwYzg5N2Y0MDYzN2Y1NmFkNDdjYmY2NTY3YTk5NmI0YjYyIiwidGFyYmFsbCI6Imh0dHBzOi8vcmVnaXN0cnkubnBtanMub3JnL3ZpemUvLS92aXplLTAuNjQ5LjAudGd6IiwiZmlsZUNvdW50Ijo4NCwiaW50ZWdyaXR5Ijoic2hhNTEyLURoTXVrUnFVY0xaZC9yejg5WGZCb2lIcmdvS3VJZzBnWmVnczFRemdFTFNBdFJ6YWphRlJIdnNsVVcyZWc0RVV6cWtWNlg1Sk5KdnZwalBqLzlLbTN3PT0iLCJzaWduYXR1cmVzIjpbeyJzaWciOiJNRVFDSUhKdTc2ZnJhRDJiL0hxSXFBN0ZlVmpybHdVUVVENU9QM1ZWU2IxaGZKUjJBaUFIMGxFOU4ybEx3NjdEQUYxNytWeE5zYW41OFpzcmRBZ0R5SlRoM0c4dkN3PT0iLCJrZXlpZCI6IlNIQTI1NjpEaFE4d1I1QVBCdkZITEYvK1RjK0FZdlBPZFRwY0lEcU9oeHNCSFJ3QzdVIn0seyJrZXlpZCI6IlNIQTI1NjpEaFE4d1I1QVBCdkZITEYvK1RjK0FZdlBPZFRwY0lEcU9oeHNCSFJ3QzdVIiwic2lnIjoiTUVVQ0lRQ2Y0MHNQblJmdEVWOWU1RVplTnNiQ1Vsd2VmRkFhcXRRNisvQjBMaENjN3dJZ1FNelplNmRYdlVjSG5kVUd3YmQ5NzJKOG1vTGFVMDArdE1vRUdXQ29RNGc9In1dLCJhdHRlc3RhdGlvbnMiOnsidXJsIjoiaHR0cHM6Ly9yZWdpc3RyeS5ucG1qcy5vcmcvLS9ucG0vdjEvYXR0ZXN0YXRpb25zL3ZpemVAMC42NDkuMCIsInByb3ZlbmFuY2UiOnsicHJlZGljYXRlVHlwZSI6Imh0dHBzOi8vc2xzYS5kZXYvcHJvdmVuYW5jZS92MSJ9fSwidW5wYWNrZWRTaXplIjo4MjQxNzR9LCJlbmdpbmVzIjp7Im5vZGUiOiI+PTIyIn0sInBlZXJEZXBlbmRlbmNpZXNNZXRhIjp7InZ1ZSI6eyJvcHRpb25hbCI6dHJ1ZX0sIkBwa2wtY29tbXVuaXR5L3BrbCI6eyJvcHRpb25hbCI6dHJ1ZX19fQ==";
const memberSha256 = "d4aed4dd2ad66b02812398b48097c3f02df1c3ff5d37d0c00c2a9c8fa89e5296";
const memberRaw = Buffer.from(memberBase64, "base64");
assert.equal(sha256(memberRaw), memberSha256);
const member = asRecord(JSON.parse(memberRaw.toString("utf8")));
const dist = asRecord(member.dist);
const lock = { version: "0.649.0", integrity: dist.integrity, resolved: dist.tarball };
const capsule = {
  integrity: dist.integrity,
  tarball: dist.tarball,
  metadataOrigin: e649MetadataUrl,
};
const packument = () => ({ name: "vize", versions: { "0.649.0": structuredClone(member) } });
const response = (raw: Buffer, status = 200) => {
  const value = new Response(Uint8Array.from(raw), {
    status,
    headers: { "content-type": "application/vnd.npm.install-v1+json" },
  });
  Object.defineProperty(value, "url", { value: e649MetadataUrl });
  return value;
};
async function temporary(run: (root: string) => Promise<void>) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "q16-e649-metadata-")));
  try {
    await run(root);
  } finally {
    rmSync(root, { recursive: true });
  }
}
test("abbreviated metadata choice is closed to the actual E649 vize package", () => {
  const profile = profileFor();
  assert(usesE649Packument(profile, "vize", "0.649.0"));
  assert(!usesE649Packument(profile, "@vizejs/native", "0.649.0"));
  assert(!usesE649Packument(profile, "vize", "0.650.0"));
  assert(!usesE649Packument({ ...profile, version: "0.650.0" }, "vize", "0.649.0"));
  assert(!usesE649Packument({ ...profile, subset: "D61", fix: profile.source }, "vize", "0.649.0"));
});
test("whole genuine selected member binds identity SRI tar and provenance URL without canonical-status invention", () => {
  assert.deepEqual(selectE649Metadata(packument(), lock, capsule), member);
  for (const mutate of [
    (value: ReturnType<typeof packument>) => {
      value.name = "foreign";
    },
    (value: ReturnType<typeof packument>) => {
      Reflect.deleteProperty(value.versions, "0.649.0");
    },
    (value: ReturnType<typeof packument>) => {
      value.versions["0.649.0"].name = "foreign";
    },
    (value: ReturnType<typeof packument>) => {
      value.versions["0.649.0"].version = "0.648.0";
    },
    (value: ReturnType<typeof packument>) => {
      asRecord(value.versions["0.649.0"].dist).integrity =
        "sha512-" + Buffer.alloc(64).toString("base64");
    },
    (value: ReturnType<typeof packument>) => {
      asRecord(value.versions["0.649.0"].dist).tarball =
        "https://registry.npmjs.org/vize/-/vize-0.648.0.tgz";
    },
    (value: ReturnType<typeof packument>) => {
      asRecord(asRecord(value.versions["0.649.0"].dist).attestations).url =
        "https://registry.npmjs.org/-/npm/v1/attestations/vize@0.648.0";
    },
  ]) {
    const value = packument();
    mutate(value);
    assert.throws(() => selectE649Metadata(value, lock, capsule), assert.AssertionError);
  }
  assert.throws(
    () =>
      selectE649Metadata(
        packument(),
        { ...lock, resolved: "https://registry.npmjs.org/foreign.tgz" },
        capsule,
      ),
    assert.AssertionError,
  );
  assert.throws(
    () =>
      selectE649Metadata(packument(), lock, {
        ...capsule,
        metadataOrigin: e649MetadataUrl + "/0.649.0",
      }),
    assert.AssertionError,
  );
});
test("actual reader seam retains whole bytes and uses one exact same-run request for both projects", () =>
  temporary(async (root) => {
    const bytes = Buffer.from(JSON.stringify(packument()));
    const cache = new Map<string, Buffer>();
    let calls = 0;
    const fetchMock: typeof fetch = async (url, options) => {
      calls++;
      assert.equal(url, e649MetadataUrl);
      assert.equal(options?.redirect, "error");
      assert.deepEqual(options?.headers, {
        Accept: e649MetadataAccept,
        "User-Agent": "VizeReleaseVerification/1 (https://github.com/ubugeeei-prod/vize)",
      });
      return response(bytes);
    };
    for (let index = 0; index < 2; index++) {
      const directory = join(root, String(index));
      mkdirSync(directory);
      assert.deepEqual(
        await readE649Metadata(profileFor(), lock, capsule, recorder(directory), cache, fetchMock),
        member,
      );
      assert(readFileSync(join(directory, "vize-install-packument.raw.json")).equals(bytes));
      const selection = asRecord(
        JSON.parse(readFileSync(join(directory, "vize-install-packument-selection.json"), "utf8")),
      );
      assert.equal(selection.canonicalVersionHttpStatusClaimed, false);
      assert.deepEqual(selection.selectedEntry, member);
    }
    assert.equal(calls, 1);
  }));
test("non-200 refusal retains the actual complete body and never populates successful cache", () =>
  temporary(async (root) => {
    const bytes = Buffer.from('{"error":"not found"}');
    const cache = new Map<string, Buffer>();
    const fetchMock: typeof fetch = async () => response(bytes, 404);
    await assert.rejects(
      readE649Metadata(profileFor(), lock, capsule, recorder(root), cache, fetchMock),
      assert.AssertionError,
    );
    assert(readFileSync(join(root, "vize-install-packument.raw.json")).equals(bytes));
    assert.equal(cache.size, 0);
  }));
