// Public npm metadata and SLSA payload binding, without signature verification.
import { isDeepStrictEqual } from "node:util";
import { CustodyError, lookup, type Source } from "./identity.ts";

const repository = "https://github.com/ubugeeei-prod/vize";

export type Reader = (url: string) => Promise<unknown>;

/** Python urlsplit components: scheme, netloc, path, query, fragment. */
export function publicUrl(url: string): string {
  const match = /^([A-Za-z][A-Za-z0-9+.-]*):\/\/([^/?#]*)([^?#]*)(?:\?([^#]*))?(?:#(.*))?$/u.exec(
    url,
  );
  if (
    !match ||
    match[1]?.toLowerCase() !== "https" ||
    match[2] !== "registry.npmjs.org" ||
    !(match[3] ?? "").startsWith("/") ||
    match[4] ||
    match[5]
  )
    throw new CustodyError("public npm registry URL required");
  return url;
}

export async function readBytes(url: string): Promise<Buffer> {
  const response = await fetch(publicUrl(url), {
    headers: { "User-Agent": "Vize-release-verification/1.0", Accept: "application/json" },
    signal: AbortSignal.timeout(30_000),
  });
  publicUrl(response.url);
  if (response.status !== 200) throw new CustodyError("public registry HTTP status differs");
  return Buffer.from(await response.arrayBuffer());
}

export async function read(url: string): Promise<unknown> {
  return JSON.parse((await readBytes(url)).toString("utf8"));
}

const base64Pattern = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/u;

/** Strict base64 decoding, like Python's `b64decode(..., validate=True)`. */
function decodeBase64(text: string): Buffer {
  if (!base64Pattern.test(text)) throw new CustodyError("invalid base64");
  return Buffer.from(text, "base64");
}

export function sriSha512(integrity: unknown): Buffer {
  if (typeof integrity !== "string" || !/^sha512-[A-Za-z0-9+/]+={0,2}$/u.test(integrity))
    throw new CustodyError("one exact SHA512 SRI required");
  const raw = decodeBase64(integrity.slice(7));
  if (raw.length !== 64 || raw.toString("base64") !== integrity.slice(7))
    throw new CustodyError("canonical SHA512 SRI required");
  return raw;
}

const list = (value: unknown): readonly unknown[] => (Array.isArray(value) ? value : []);
const escapeRegExp = (text: string): string => text.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");

export async function provenancePayloadBinding(
  dist: unknown,
  source: Pick<Source, "H" | "tag" | "R">,
  reader: Reader = read,
): Promise<Record<string, unknown>> {
  const attestationUrl = lookup(dist, "attestations", "url");
  const url = publicUrl(typeof attestationUrl === "string" ? attestationUrl : "");
  const expected = sriSha512(lookup(dist, "integrity")).toString("hex");
  const invocationPattern = new RegExp(
    `^${escapeRegExp(`${repository}/actions/runs/${source.R}`)}/attempts/[1-9][0-9]*$`,
    "u",
  );
  const candidates: Record<string, unknown>[] = [];
  for (const item of list(lookup(await reader(url), "attestations"))) {
    let payload: unknown;
    try {
      const encoded = lookup(item, "bundle", "dsseEnvelope", "payload");
      if (typeof encoded !== "string") continue;
      payload = JSON.parse(decodeBase64(encoded).toString("utf8"));
    } catch {
      continue;
    }
    if (lookup(payload, "predicateType") !== "https://slsa.dev/provenance/v1") continue;
    const definition = lookup(payload, "predicate", "buildDefinition");
    const workflow = lookup(definition, "externalParameters", "workflow") ?? {};
    const invocation = lookup(payload, "predicate", "runDetails", "metadata", "invocationId") ?? "";
    const commits = list(lookup(definition, "resolvedDependencies")).map(
      (dependency) => lookup(dependency, "digest", "gitCommit") ?? null,
    );
    const subjects = list(lookup(payload, "subject")).map(
      (subject) => lookup(subject, "digest", "sha512") ?? null,
    );
    const matches =
      isDeepStrictEqual(workflow, {
        ref: `refs/heads/release/${source.tag}`,
        repository,
        path: ".github/workflows/release.yml",
      }) &&
      isDeepStrictEqual(commits, [source.H]) &&
      isDeepStrictEqual(subjects, [expected]) &&
      typeof invocation === "string" &&
      invocationPattern.test(invocation);
    candidates.push({
      workflow,
      sourceCommits: commits,
      invocation,
      subjectIntegrity: isDeepStrictEqual(subjects, [expected]),
      matches,
    });
  }
  if (candidates.length !== 1 || candidates[0]?.matches !== true)
    throw new CustodyError("public provenance payload source/run/subject binding differs");
  return {
    url,
    payloads: candidates,
    payloadBindingChecked: true,
    cryptographicSignatureVerificationPerformed: false,
  };
}

/** Python `urllib.parse.quote(name, safe="")`. */
const quote = (name: string): string =>
  encodeURIComponent(name).replace(
    /[!'()*]/gu,
    (c) => `%${c.charCodeAt(0).toString(16).toUpperCase()}`,
  );

export async function packageMetadata(
  name: string,
  version: string,
  entry: unknown,
  source?: Pick<Source, "H" | "tag" | "R">,
  reader: Reader = read,
): Promise<Record<string, unknown>> {
  const resolved = lookup(entry, "resolved");
  publicUrl(typeof resolved === "string" ? resolved : "");
  sriSha512(lookup(entry, "integrity"));
  const metadata = await reader(`https://registry.npmjs.org/${quote(name)}/${version}`);
  const dist = lookup(metadata, "dist") ?? {};
  if (
    lookup(metadata, "name") !== name ||
    lookup(metadata, "version") !== version ||
    lookup(dist, "integrity") !== lookup(entry, "integrity") ||
    lookup(dist, "tarball") !== resolved
  )
    throw new CustodyError(`public registry/lock package identity differs: ${name}`);
  return source
    ? provenancePayloadBinding(dist, source, reader)
    : { scope: "third-party Corsa public archive; no Vize source/run provenance" };
}
