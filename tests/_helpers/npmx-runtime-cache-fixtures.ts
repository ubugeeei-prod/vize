import * as fs from "node:fs";

export function patchNpmxRuntimeFixtureCache(cachePath: string): void {
  let source = fs.readFileSync(cachePath, "utf-8");
  if (source.includes("vize e2e deterministic fixture-only endpoints")) {
    return;
  }

  source = replaceOnce(
    source,
    `  // npm API: downloads range → synthetic daily data for sparklines
  if (host === 'api.npmjs.org') {
`,
    `  // vize e2e deterministic fixture-only endpoints requested by package pages.
  if (
    host === 'npmx-likes-leaderboard-api-production.up.railway.app' &&
    pathname === '/api/leaderboard/likes'
  ) {
    return {
      data: {
        leaderBoard: [
          { subjectRef: 'https://npmx.dev/package/vue', totalLikes: 980 },
          { subjectRef: 'https://npmx.dev/package/nuxt', totalLikes: 760 },
          { subjectRef: 'https://npmx.dev/package/%40vue%2Fcompiler-sfc', totalLikes: 420 },
        ],
      },
    }
  }

  // Microlink enrichment metadata → deterministic empty preview metadata.
  if (host === 'api.microlink.io') {
    return {
      data: {
        data: {
          image: null,
          logo: null,
        },
      },
    }
  }

  // Vercel speed insights proxy → no-op script for local visual tests.
  if (host === 'npmx.dev' && pathname.startsWith('/_vercel/insights/')) {
    return {
      data: '/* vize e2e speed-insights noop */',
    }
  }

  // npm API: version download distribution → synthetic weekly version downloads.
  if (host === 'api.npmjs.org') {
    const versionDownloadsMatch = decodeURIComponent(pathname).match(
      /^\\/versions\\/(.+)\\/last-week$/,
    )
    if (versionDownloadsMatch?.[1]) {
      return {
        data: {
          downloads: {
            '3.5.28': 640_000,
            '3.5.29': 880_000,
            '4.0.0': 420_000,
          },
        },
      }
    }
  }

  // npm API: downloads range → synthetic daily data for sparklines
  if (host === 'api.npmjs.org') {
`,
    cachePath,
  );
  fs.writeFileSync(cachePath, source);
}

function replaceOnce(
  source: string,
  search: string,
  replacement: string,
  filePath: string,
): string {
  if (!source.includes(search)) {
    throw new Error(`missing npmx runtime cache fixture patch anchor in ${filePath}`);
  }
  return source.replace(search, replacement);
}
