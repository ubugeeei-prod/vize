import { publicReader } from "./http.ts";
import { verifyGithub } from "./github.ts";
import type { PublicationPlan, PublicationTarget } from "./plan.ts";
import { readMarketplace } from "../marketplace_query.ts";

export interface PublicationOptions {
  tag: string;
  cut: string;
  head: string;
  sourcePr: string | number;
  run: string | number;
  fetch?: typeof globalThis.fetch;
}

type ObjectValue = Record<string, unknown>;
const repository = "ubugeeei-prod/vize";

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function object(value: unknown): ObjectValue {
  requireValue(
    value && typeof value === "object" && !Array.isArray(value),
    "registry object required",
  );
  return value as ObjectValue;
}

function publicUrl(value: unknown, host: string): string {
  requireValue(typeof value === "string", "public download URL required");
  const url = new URL(value);
  requireValue(
    url.protocol === "https:" &&
      url.hostname === host &&
      !url.username &&
      !url.password &&
      !url.port &&
      !url.hash,
    "unexpected public download host",
  );
  return url.href;
}

/** Public registry identity and archive digests, not signature verification or installed-product proof. */
export async function verifyPublication(plan: PublicationPlan, options: PublicationOptions) {
  const run = String(options.run);
  const sourcePr = String(options.sourcePr);
  requireValue(
    /^[0-9a-f]{40}$/.test(options.cut) && plan.parentCut === options.cut,
    "public source C mismatch",
  );
  requireValue(
    /^[0-9a-f]{40}$/.test(options.head) && plan.head === options.head,
    "public source H mismatch",
  );
  requireValue(
    /^v0\.(0|[1-9][0-9]*)\.0$/.test(options.tag) && options.tag === `v${plan.version}`,
    "public source tag/version mismatch",
  );
  requireValue(
    /^[1-9][0-9]*$/.test(run) &&
      (typeof options.run !== "number" || Number.isSafeInteger(options.run)),
    "positive Release run required",
  );
  requireValue(
    /^[1-9][0-9]*$/.test(sourcePr) &&
      (typeof options.sourcePr !== "number" || Number.isSafeInteger(options.sourcePr)),
    "positive source PR required",
  );
  for (const [kind, targets] of [
    ["npm", plan.npm],
    ["crate", plan.crates],
  ] as const) {
    requireValue(
      targets.length > 0 && new Set(targets.map((item) => item.name)).size === targets.length,
      `nonempty distinct ${kind} targets required`,
    );
    requireValue(
      targets.every(
        (item) =>
          item.version === plan.version &&
          (kind === "npm" ? /^(?:@[a-z0-9-]+\/)?[a-z0-9-]+$/ : /^[a-z0-9_]+$/).test(item.name),
      ),
      `exact ${kind} source versions required`,
    );
  }
  requireValue(
    plan.editor.version === plan.version &&
      /^[a-z0-9-]+$/.test(plan.editor.publisher) &&
      /^[a-z0-9-]+$/.test(plan.editor.name),
    "exact editor source identity required",
  );
  const { json, download, text } = publicReader(options.fetch);

  const { tagObject, tag, source, releaseRun, release, jobs, integrationPr } = await verifyGithub(
    options,
    json,
  );

  const npmTarget = async (item: PublicationTarget) => {
    const metadataUrl = `https://registry.npmjs.org/${encodeURIComponent(item.name)}/${item.version}`;
    const metadata = await json(metadataUrl);
    requireValue(
      metadata.name === item.name && metadata.version === item.version,
      `npm exact version mismatch: ${item.name}`,
    );
    const dist = object(metadata.dist);
    const url = publicUrl(dist.tarball, "registry.npmjs.org");
    requireValue(
      typeof dist.integrity === "string" && /^sha512-[A-Za-z0-9+/]{86}==$/.test(dist.integrity),
      `npm SHA512 SRI required: ${item.name}`,
    );
    const hashes = await download(url);
    requireValue(dist.integrity === `sha512-${hashes.sha512}`, `npm SRI mismatch: ${item.name}`);
    return {
      ...item,
      metadataUrl,
      url,
      tarball: url,
      integrity: dist.integrity,
      sha256: hashes.sha256,
      bytes: hashes.bytes,
    };
  };
  const crateTarget = async (item: PublicationTarget) => {
    const metadataUrl = `https://crates.io/api/v1/crates/${item.name}/${item.version}`;
    const metadata = object((await json(metadataUrl)).version);
    requireValue(
      metadata.crate === item.name &&
        metadata.num === item.version &&
        metadata.yanked === false &&
        typeof metadata.checksum === "string" &&
        /^[0-9a-f]{64}$/.test(metadata.checksum),
      `crate exact version/non-yanked checksum mismatch: ${item.name}`,
    );
    const url = `https://static.crates.io/crates/${item.name}/${item.name}-${item.version}.crate`;
    const hashes = await download(url);
    requireValue(
      hashes.sha256 === metadata.checksum,
      `crate archive checksum mismatch: ${item.name}`,
    );
    return {
      ...item,
      metadataUrl,
      url,
      checksum: metadata.checksum,
      sha256: hashes.sha256,
      bytes: hashes.bytes,
      yanked: false,
    };
  };
  // Fixed bounded concurrency also keeps native archive hashing memory constant.
  const collect = async <T>(
    items: PublicationTarget[],
    probe: (item: PublicationTarget) => Promise<T>,
  ) => {
    const results: T[] = [];
    for (let index = 0; index < items.length; index += 3) {
      results.push(...(await Promise.all(items.slice(index, index + 3).map(probe))));
    }
    return results;
  };
  const npm = await collect(plan.npm, npmTarget);
  const crates = await collect(plan.crates, crateTarget);

  const editor = plan.editor;
  const { extension: marketplace, ...marketplaceObservation } = await readMarketplace(
    `${editor.publisher}.${editor.name}`,
    options.fetch,
  );
  requireValue(
    object(marketplace.publisher).publisherName === editor.publisher &&
      marketplace.extensionName === editor.name &&
      Array.isArray(marketplace.versions) &&
      marketplace.versions.some((entry) => object(entry).version === editor.version),
    "Marketplace exact version missing",
  );
  const openVsxUrl = `https://open-vsx.org/api/${editor.publisher}/${editor.name}/${editor.version}`;
  const openVsx = await (async () => {
    try {
      const metadata = await json(openVsxUrl);
      requireValue(
        metadata.namespace === editor.publisher &&
          metadata.name === editor.name &&
          metadata.version === editor.version,
        "Open VSX exact version missing",
      );
      return {
        optional: true,
        status: "available",
        download: publicUrl(object(metadata.files).download, "open-vsx.org"),
      };
    } catch (error) {
      const reason = String(error).slice(0, 600);
      return {
        optional: true,
        status: /HTTP 404\b|exact version missing/.test(reason) ? "missing" : "error",
        reason,
      };
    }
  })();
  const githubAssets = [];
  for (const name of plan.githubAssets) {
    requireValue(
      name === "zed-vize-extension.tar.gz" && Array.isArray(release.assets),
      "planned own GitHub editor asset required",
    );
    const assetUrl = (assetName: string) => {
      const matches = (release.assets as unknown[])
        .map(object)
        .filter((asset) => asset.name === assetName && asset.state === "uploaded");
      const expected = `https://github.com/${repository}/releases/download/${options.tag}/${assetName}`;
      requireValue(
        matches.length === 1 && matches[0].browser_download_url === expected,
        "own GitHub Release asset identity mismatch",
      );
      return expected;
    };
    const url = assetUrl(name),
      checksumUrl = assetUrl(`${name}.sha256`);
    const hashes = await download(url, "follow");
    const checksum = await text(checksumUrl, "follow");
    requireValue(
      checksum.trim() === `${hashes.sha256}  ${name}`,
      "GitHub editor asset checksum mismatch",
    );
    githubAssets.push({ name, url, checksumUrl, sha256: hashes.sha256, bytes: hashes.bytes });
  }
  return {
    schema: "vize-public-release-acceptance-v1",
    checkedAt: new Date().toISOString(),
    source: {
      cut: options.cut,
      head: options.head,
      sourcePr,
      tag: options.tag,
      run,
      authority: plan.authority,
    },
    github: {
      tagObject: tagObject.sha,
      target: options.head,
      releaseId: release.id,
      targetCommitish: release.target_commitish,
      publishedAt: release.published_at,
      runStatus: releaseRun.status,
      runConclusion: releaseRun.conclusion,
      runTitle: releaseRun.display_title,
      annotation: tag.message,
      sourceState: source.state,
      integrationPr,
      jobs,
    },
    npm,
    crates,
    githubAssets,
    marketplace: {
      ...editor,
      metadataUrl: marketplaceObservation.url,
      ...marketplaceObservation,
      exactVersionAvailable: true,
    },
    openVsx: { ...editor, metadataUrl: openVsxUrl, ...openVsx },
    evidence:
      "Public metadata and downloaded npm/crate digest verification; no signature or installed-product execution claim.",
    success: true,
  };
}
