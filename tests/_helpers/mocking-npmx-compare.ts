import type { Page, Route } from "@playwright/test";

type NpmxComparePackageName = "react" | "vue";

interface NpmxCompareFixture {
  dependencies: Record<string, string>;
  downloads: number;
  forks: number;
  issues: number;
  likes: number;
  packageSize: number;
  repository: { owner: string; repo: string; url: string };
  stars: number;
  totalSize: number;
  version: string;
}

const NPMX_COMPARE_FIXTURES: Record<NpmxComparePackageName, NpmxCompareFixture> = {
  vue: {
    dependencies: {},
    downloads: 8_400_000,
    forks: 35_000,
    issues: 590,
    likes: 980,
    packageSize: 2_600_000,
    repository: {
      owner: "vuejs",
      repo: "core",
      url: "https://github.com/vuejs/core",
    },
    stars: 52_000,
    totalSize: 5_400_000,
    version: "3.5.29",
  },
  react: {
    dependencies: { "loose-envify": "^1.1.0" },
    downloads: 6_900_000,
    forks: 49_000,
    issues: 870,
    likes: 720,
    packageSize: 420_000,
    repository: {
      owner: "facebook",
      repo: "react",
      url: "https://github.com/facebook/react",
    },
    stars: 235_000,
    totalSize: 1_100_000,
    version: "19.1.0",
  },
};

function normalizeNpmxComparePackageName(rawName: string): NpmxComparePackageName | null {
  const name = decodeURIComponent(rawName)
    .replace(/^@?npm\//, "")
    .toLowerCase();
  return name === "vue" || name === "react" ? name : null;
}

function npmxComparePackageNameFromUrl(url: URL): NpmxComparePackageName | null {
  const pathname = decodeURIComponent(url.pathname);
  const segment = pathname.split("/").filter(Boolean).at(-1);
  return segment ? normalizeNpmxComparePackageName(segment) : null;
}

function npmxCompareFixture(rawName: string): [NpmxComparePackageName, NpmxCompareFixture] | null {
  const name = normalizeNpmxComparePackageName(rawName);
  return name ? [name, NPMX_COMPARE_FIXTURES[name]] : null;
}

function npmxComparePackument(name: NpmxComparePackageName, fixture: NpmxCompareFixture) {
  return {
    name,
    license: "MIT",
    repository: {
      type: "git",
      url: `git+${fixture.repository.url}.git`,
    },
    "dist-tags": {
      latest: fixture.version,
    },
    time: {
      created: "2020-01-01T00:00:00.000Z",
      modified: "2026-01-01T00:00:00.000Z",
      [fixture.version]: "2025-12-15T00:00:00.000Z",
    },
    versions: {
      [fixture.version]: {
        name,
        version: fixture.version,
        license: "MIT",
        main: "index.js",
        module: "dist/index.mjs",
        types: "dist/index.d.ts",
        dependencies: fixture.dependencies,
        engines: {
          node: ">=18",
        },
        repository: {
          type: "git",
          url: `git+${fixture.repository.url}.git`,
        },
        dist: {
          tarball: `https://registry.npmjs.org/${name}/-/${name}-${fixture.version}.tgz`,
          unpackedSize: fixture.packageSize,
        },
      },
    },
  };
}

function npmxCompareAnalysis(name: NpmxComparePackageName, fixture: NpmxCompareFixture) {
  return {
    package: name,
    version: fixture.version,
    moduleFormat: "dual",
    types: { kind: "included" },
    engines: { node: ">=18" },
    devDependencySuggestion: {
      kind: "runtime",
      confidence: "high",
      reasons: [],
    },
  };
}

function npmxCompareVulnerabilities(name: NpmxComparePackageName, fixture: NpmxCompareFixture) {
  return {
    package: name,
    version: fixture.version,
    vulnerablePackages: [],
    deprecatedPackages: [],
    totalPackages: 1 + Object.keys(fixture.dependencies).length,
    failedQueries: 0,
    totalCounts: {
      total: 0,
      critical: 0,
      high: 0,
      moderate: 0,
      low: 0,
    },
  };
}

async function fulfillJson(route: Route, body: unknown): Promise<void> {
  await route.fulfill({
    status: 200,
    contentType: "application/json",
    body: JSON.stringify(body),
  });
}
/**
 * Make the npmx `/compare?packages=vue,react` route deterministic for visual
 * parity. The route fetches package comparison data after hydration, so these
 * context routes stabilize both the reference Vue page and candidate vize page.
 */
export async function setupNpmxCompareMocks(page: Page): Promise<void> {
  await page.context().route("https://registry.npmjs.org/**", async (route) => {
    const requestUrl = new URL(route.request().url());
    const rawName = requestUrl.pathname.split("/").filter(Boolean).join("/");
    const fixture = npmxCompareFixture(rawName);
    if (!fixture) return route.fallback();

    await fulfillJson(route, npmxComparePackument(...fixture));
  });

  await page.context().route("https://api.npmjs.org/downloads/point/**", async (route) => {
    const name = npmxComparePackageNameFromUrl(new URL(route.request().url()));
    if (!name) return route.fallback();

    await fulfillJson(route, {
      downloads: NPMX_COMPARE_FIXTURES[name].downloads,
      package: name,
    });
  });

  await page.context().route("https://ungh.cc/repos/**", async (route) => {
    const requestUrl = new URL(route.request().url());
    const [, owner, repo] = requestUrl.pathname.match(/^\/repos\/([^/]+)\/([^/]+)/) ?? [];
    const fixture = Object.values(NPMX_COMPARE_FIXTURES).find(
      (entry) => entry.repository.owner === owner && entry.repository.repo === repo,
    );
    if (!fixture) return route.fallback();

    await fulfillJson(route, {
      repo: {
        stars: fixture.stars,
        forks: fixture.forks,
      },
    });
  });

  await page.context().route("**/api/registry/analysis/**", async (route) => {
    const name = npmxComparePackageNameFromUrl(new URL(route.request().url()));
    if (!name) return route.fallback();

    await fulfillJson(route, npmxCompareAnalysis(name, NPMX_COMPARE_FIXTURES[name]));
  });

  await page.context().route("**/api/registry/vulnerabilities/**", async (route) => {
    const name = npmxComparePackageNameFromUrl(new URL(route.request().url()));
    if (!name) return route.fallback();

    await fulfillJson(route, npmxCompareVulnerabilities(name, NPMX_COMPARE_FIXTURES[name]));
  });

  await page.context().route("**/api/social/likes/**", async (route) => {
    const name = npmxComparePackageNameFromUrl(new URL(route.request().url()));
    if (!name) return route.fallback();

    await fulfillJson(route, {
      totalLikes: NPMX_COMPARE_FIXTURES[name].likes,
      userHasLiked: false,
      topLikedRank: null,
    });
  });

  await page.context().route("**/api/github/issues/**", async (route) => {
    const requestUrl = new URL(route.request().url());
    const [, owner, repo] =
      requestUrl.pathname.match(/\/api\/github\/issues\/([^/]+)\/([^/]+)/) ?? [];
    const fixture = Object.values(NPMX_COMPARE_FIXTURES).find(
      (entry) => entry.repository.owner === owner && entry.repository.repo === repo,
    );
    if (!fixture) return route.fallback();

    await fulfillJson(route, {
      owner,
      repo,
      issues: fixture.issues,
    });
  });

  await page.context().route("**/api/registry/install-size/**", async (route) => {
    const name = npmxComparePackageNameFromUrl(new URL(route.request().url()));
    if (!name) return route.fallback();
    const fixture = NPMX_COMPARE_FIXTURES[name];

    await fulfillJson(route, {
      selfSize: fixture.packageSize,
      totalSize: fixture.totalSize,
      dependencyCount: Object.keys(fixture.dependencies).length,
    });
  });
}
