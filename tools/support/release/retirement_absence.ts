import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { derivePublicationPlan, type PublicationPlan } from "./public_acceptance/plan.ts";

import {
  absenceEvidence,
  absenceResponseLimit,
  type AbsenceTarget as Target,
  type AbsenceEvidence,
} from "./retirement_evidence.ts";
export type AbsenceOptions = { fetch?: typeof globalThis.fetch };
export type AbsenceObservation = Target & {
  status: 200 | 404;
  evidence: AbsenceEvidence;
  observedAt: string;
  responseSha256: string;
  responseBytes: number;
  response: Record<string, unknown>;
  date: string | null;
};

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function object(value: unknown): Record<string, unknown> {
  requireValue(
    value && typeof value === "object" && !Array.isArray(value),
    "missing-response object required",
  );
  return value as Record<string, unknown>;
}

function targets(plan: PublicationPlan): Target[] {
  requireValue(/^[a-f0-9]{40}$/.test(plan.head), "full source H required");
  requireValue(/^[a-f0-9]{40}$/.test(plan.parentCut), "full source C required");
  requireValue(
    /^0\.(0|[1-9][0-9]*)\.0$/.test(plan.version),
    "stable minor source version required",
  );
  const result: Target[] = [];
  const append = (channel: "npm" | "crates") => {
    requireValue(Array.isArray(plan[channel]) && plan[channel].length > 0, `empty ${channel} plan`);
    const names = new Set<string>();
    for (const target of plan[channel]) {
      requireValue(
        target.version === plan.version && !names.has(target.name),
        `duplicate/version ${channel} target`,
      );
      const valid =
        channel === "npm"
          ? /^(@vizejs\/[a-z0-9-]+|vize|oxlint-plugin-vize)$/
          : /^vize(?:_[a-z0-9_]+)?$/;
      requireValue(valid.test(target.name), `unsafe ${channel} target`);
      names.add(target.name);
      result.push({
        channel,
        ...target,
        url:
          channel === "npm"
            ? `https://registry.npmjs.org/${encodeURIComponent(target.name)}/${target.version}`
            : `https://crates.io/api/v1/crates/${target.name}/${target.version}`,
      });
    }
  };
  append("npm");
  append("crates");
  const editor = plan.editor;
  requireValue(
    /^[a-z0-9-]+$/.test(editor.publisher) &&
      /^[a-z0-9-]+$/.test(editor.name) &&
      editor.version === plan.version,
    "unsafe source editor identity/version",
  );
  const name = `${editor.publisher}.${editor.name}`;
  result.push(
    {
      channel: "marketplace",
      name,
      version: editor.version,
      url: `https://marketplace.visualstudio.com/_apis/gallery/publishers/${editor.publisher}/extensions/${editor.name}?flags=1&api-version=7.2-preview.2`,
    },
    {
      channel: "openvsx",
      name,
      version: editor.version,
      url: `https://open-vsx.org/api/${editor.publisher}/${editor.name}/${editor.version}`,
    },
  );
  return result;
}

async function responseBytes(response: Response, limit: number): Promise<Buffer> {
  const declared = response.headers.get("content-length");
  requireValue(
    declared === null || (/^\d+$/.test(declared) && Number(declared) <= limit),
    `absence response exceeds ${limit} bytes`,
  );
  requireValue(response.body, "missing absence response body");
  const reader = response.body.getReader();
  const chunks: Buffer[] = [];
  let length = 0;
  try {
    for (;;) {
      const part = await reader.read();
      if (part.done) break;
      length += part.value.byteLength;
      requireValue(length <= limit, `absence response exceeds ${limit} bytes`);
      chunks.push(Buffer.from(part.value));
    }
  } finally {
    void reader.cancel().catch(() => {});
    reader.releaseLock();
  }
  requireValue(length > 0, "empty absence response");
  return Buffer.concat(chunks, length);
}

async function observe(
  target: Target,
  fetchImpl: typeof globalThis.fetch,
): Promise<AbsenceObservation> {
  const controller = new AbortController();
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => {
      controller.abort();
      reject(new Error(`absence request timed out after 30000ms: ${target.url}`));
    }, 30_000);
  });
  try {
    return await Promise.race([
      timeout,
      (async () => {
        const response = await fetchImpl(target.url, {
          method: "GET",
          credentials: "omit",
          redirect: "manual",
          cache: "no-store",
          signal: controller.signal,
          headers: {
            Accept: "application/json",
            "Cache-Control": "no-cache",
            Pragma: "no-cache",
            "User-Agent": "vize-retired-unpublished-cut",
          },
        });
        requireValue(
          !response.redirected && response.url === target.url,
          "absence response URL changed",
        );
        requireValue(
          response.status === 404 || (target.channel === "marketplace" && response.status === 200),
          `absence requires HTTP 404 or complete Marketplace inventory, received ${response.status}: ${target.url}`,
        );
        requireValue(!response.headers.has("location"), "absence redirects refused");
        requireValue(
          /^application\/(?:json|[a-z0-9.+-]+\+json)(?:;|$)/i.test(
            response.headers.get("content-type") ?? "",
          ),
          "absence JSON content type required",
        );
        const bytes = await responseBytes(response, absenceResponseLimit(target, response.status));
        const body = object(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)));
        let evidence: AbsenceEvidence;
        try {
          evidence = absenceEvidence(target, response.status, body, response.headers);
        } catch (error) {
          throw new Error(
            `${String(error)}; HTTP ${response.status} at ${target.url}; response sha256=${createHash("sha256").update(bytes).digest("hex")}; bytes=${bytes.length}; body=${bytes.toString("utf8")}`,
          );
        }
        return {
          ...target,
          status: response.status as 200 | 404,
          evidence,
          observedAt: new Date().toISOString(),
          responseSha256: createHash("sha256").update(bytes).digest("hex"),
          responseBytes: bytes.length,
          response: body,
          date: response.headers.get("date"),
        };
      })(),
    ]);
  } finally {
    clearTimeout(timer);
    controller.abort();
  }
}

/** Pure read-only controls are injectable only through this test API, never CLI/env inputs. */
export async function verifyRetirementAbsence(plan: PublicationPlan, options: AbsenceOptions = {}) {
  const planned = targets(plan);
  const observations: AbsenceObservation[] = [];
  // Bound public requests to three concurrently; stop issuing batches on the first refusal.
  for (let offset = 0; offset < planned.length; offset += 3) {
    observations.push(
      ...(await Promise.all(
        planned
          .slice(offset, offset + 3)
          .map((target) => observe(target, options.fetch ?? globalThis.fetch)),
      )),
    );
  }
  return {
    schema: "vize-retired-unpublished-registry-absence-v1",
    observedAt: new Date().toISOString(),
    head: plan.head,
    tag: `v${plan.version}`,
    plan,
    observations,
    scope:
      "Fresh public exact-version absence via typed missing responses or complete Marketplace version inventory; GitHub ref/Release and failed-run custody are verified separately.",
  };
}

export async function verifyRetirementSourceAbsence(
  root: string,
  head: string,
  tag: string,
  options: AbsenceOptions = {},
) {
  const plan = derivePublicationPlan(root, head);
  requireValue(tag === `v${plan.version}`, "retirement tag differs from raw H version");
  return verifyRetirementAbsence(plan, options);
}

/** Only explicit root/H/tag/output arguments; an existing output may not be reused. */
export async function retirementAbsenceMain(args: string[], options: AbsenceOptions = {}) {
  requireValue(args.length === 8, "expected --root ROOT --head H --tag TAG --output PATH");
  const values = new Map<string, string>();
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index],
      value = args[index + 1];
    requireValue(
      ["--root", "--head", "--tag", "--output"].includes(key) &&
        !values.has(key) &&
        value.length > 0,
      "invalid retirement absence arguments",
    );
    values.set(key, value);
  }
  requireValue(values.size === 4, "four retirement absence arguments required");
  const receipt = await verifyRetirementSourceAbsence(
    values.get("--root")!,
    values.get("--head")!,
    values.get("--tag")!,
    options,
  );
  writeFileSync(values.get("--output")!, `${JSON.stringify(receipt, null, 2)}\n`, {
    flag: "wx",
    mode: 0o600,
  });
  return receipt;
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  retirementAbsenceMain(process.argv.slice(2)).catch((error: unknown) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
