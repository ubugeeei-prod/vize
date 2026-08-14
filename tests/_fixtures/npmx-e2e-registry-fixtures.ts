import type { Packument, PackumentVersion } from '#shared/types/npm-registry'

type FixtureLookup<T> = { handled: true; data: T } | { handled: false }
interface CachedRegistryFixture<T> {
  data: T
  isStale: false
  cachedAt: null
}

interface PackageFixture {
  description: string
  homepage: string
  keywords: string[]
  name: string
  repository: string
  versions: Record<string, VersionFixture>
}

interface VersionFixture {
  dependencies?: Record<string, string>
  fileCount: number
  integrity: string
  unpackedSize: number
}

const PUBLISHED_AT = '2026-01-01T00:00:00.000Z'
const NPM_REGISTRY = 'https://registry.npmjs.org'

const PACKAGE_FIXTURES: Record<string, PackageFixture> = {
  vue: {
    name: 'vue',
    description: 'Fixture Vue package metadata for npmx visual parity.',
    homepage: 'https://vuejs.org/',
    keywords: ['vue', 'framework'],
    repository: 'https://github.com/vuejs/core',
    versions: {
      '3.5.28': {
        dependencies: {
          '@vue/compiler-dom': '3.5.28',
          '@vue/runtime-dom': '3.5.28',
          '@vue/shared': '3.5.28',
        },
        fileCount: 42,
        integrity: 'sha512-vize-e2e-vue-3-5-28',
        unpackedSize: 2_520_000,
      },
      '3.5.29': {
        dependencies: {
          '@vue/compiler-dom': '3.5.29',
          '@vue/runtime-dom': '3.5.29',
          '@vue/shared': '3.5.29',
        },
        fileCount: 44,
        integrity: 'sha512-vize-e2e-vue-3-5-29',
        unpackedSize: 2_600_000,
      },
    },
  },
  '@vue/compiler-sfc': {
    name: '@vue/compiler-sfc',
    description: 'Fixture Vue SFC compiler metadata for npmx visual parity.',
    homepage: 'https://github.com/vuejs/core/tree/main/packages/compiler-sfc',
    keywords: ['vue', 'compiler', 'sfc'],
    repository: 'https://github.com/vuejs/core',
    versions: {
      '3.5.29': {
        dependencies: {
          '@vue/compiler-core': '3.5.29',
          '@vue/compiler-dom': '3.5.29',
          '@vue/shared': '3.5.29',
        },
        fileCount: 31,
        integrity: 'sha512-vize-e2e-vue-compiler-sfc-3-5-29',
        unpackedSize: 1_180_000,
      },
    },
  },
  nuxt: {
    name: 'nuxt',
    description: 'Fixture Nuxt package metadata for npmx visual parity.',
    homepage: 'https://nuxt.com/',
    keywords: ['nuxt', 'vue', 'framework'],
    repository: 'https://github.com/nuxt/nuxt',
    versions: {
      '4.0.0': {
        dependencies: {
          '@nuxt/kit': '4.0.0',
          '@nuxt/schema': '4.0.0',
          nitropack: '^2.12.0',
        },
        fileCount: 118,
        integrity: 'sha512-vize-e2e-nuxt-4-0-0',
        unpackedSize: 9_400_000,
      },
    },
  },
}

const PACKUMENTS: Record<string, Packument> = Object.fromEntries(
  Object.values(PACKAGE_FIXTURES).map(fixture => [fixture.name, createPackument(fixture)]),
) as Record<string, Packument>

export function resolveVizeE2ENpmRegistryFixture<T = unknown>(
  request: string,
  baseURL = NPM_REGISTRY,
): FixtureLookup<T> {
  const url = toUrl(request, baseURL)
  if (!url || !isNpmRegistryUrl(url)) return { handled: false }

  const resolved = resolveRegistryPath(url.pathname)
  if (!resolved) return { handled: false }

  const packument = PACKUMENTS[resolved.packageName]
  if (!packument) return { handled: false }

  if (resolved.versionSpecifier) {
    const version =
      resolved.versionSpecifier === 'latest'
        ? packument['dist-tags']?.latest
        : resolved.versionSpecifier
    const manifest = version ? packument.versions[version] : undefined
    return manifest ? { handled: true, data: manifest as T } : { handled: false }
  }

  return { handled: true, data: packument as T }
}

export function resolveVizeE2ENpmRegistryCachedResponse<T = unknown>(
  request: string,
  baseURL = NPM_REGISTRY,
): FixtureLookup<CachedRegistryFixture<T>> {
  const fixture = resolveVizeE2ENpmRegistryFixture<T>(request, baseURL)
  return fixture.handled
    ? {
        handled: true,
        data: {
          data: fixture.data,
          isStale: false,
          cachedAt: null,
        },
      }
    : { handled: false }
}

export function resolveVizeE2ENpmPackument<T = Packument>(
  encodedName: string,
  baseURL = NPM_REGISTRY,
): FixtureLookup<T> {
  return resolveVizeE2ENpmRegistryFixture<T>(`/${encodedName}`, baseURL)
}

export function resolveVizeE2EFastNpmMetaFixture<T = unknown>(
  request: string,
): FixtureLookup<T> {
  const url = toUrl(request, 'https://npm.antfu.dev')
  if (!url || url.origin !== 'https://npm.antfu.dev') return { handled: false }

  const parsed = parseFastNpmMetaPath(url.pathname)
  if (!parsed) return { handled: false }

  const packument = PACKUMENTS[parsed.packageName]
  if (!packument) return { handled: false }

  const latest = packument['dist-tags']?.latest
  const version = parsed.versionSpecifier ?? latest
  if (!version || !packument.versions[version]) return { handled: false }

  return { handled: true, data: { version } as T }
}

export function resolveVizeE2EFastNpmMetaVersion(request: string): FixtureLookup<string> {
  const fixture = resolveVizeE2EFastNpmMetaFixture<{ version: string }>(request)
  return fixture.handled ? { handled: true, data: fixture.data.version } : { handled: false }
}

function createPackument(fixture: PackageFixture): Packument {
  const versionEntries = Object.entries(fixture.versions)
  const latest = versionEntries.at(-1)?.[0] ?? ''
  const versions = Object.fromEntries(
    versionEntries.map(([version, versionFixture]) => [
      version,
      createVersion(fixture, version, versionFixture),
    ]),
  )
  const time = Object.fromEntries(versionEntries.map(([version]) => [version, PUBLISHED_AT]))

  return {
    '_id': fixture.name,
    '_rev': 'vize-e2e-fixture',
    'name': fixture.name,
    'description': fixture.description,
    'dist-tags': { latest },
    'time': {
      created: PUBLISHED_AT,
      modified: PUBLISHED_AT,
      ...time,
    },
    'maintainers': [{ name: 'vize-fixture', email: 'fixtures@example.com' }],
    'author': { name: 'vize-fixture' },
    'license': 'MIT',
    'homepage': fixture.homepage,
    'keywords': fixture.keywords,
    'repository': {
      type: 'git',
      url: `git+${fixture.repository}.git`,
    },
    'bugs': {
      url: `${fixture.repository}/issues`,
    },
    'readme': `# ${fixture.name}\n\nDeterministic npmx visual parity fixture.\n`,
    'readmeFilename': 'README.md',
    'versions': versions,
  }
}

function createVersion(
  fixture: PackageFixture,
  version: string,
  versionFixture: VersionFixture,
): PackumentVersion {
  return {
    _id: `${fixture.name}@${version}`,
    _npmVersion: '11.0.0',
    name: fixture.name,
    version,
    description: fixture.description,
    license: 'MIT',
    homepage: fixture.homepage,
    keywords: fixture.keywords,
    repository: {
      type: 'git',
      url: `git+${fixture.repository}.git`,
    },
    bugs: {
      url: `${fixture.repository}/issues`,
    },
    main: 'dist/index.cjs',
    module: 'dist/index.mjs',
    types: 'dist/index.d.ts',
    dependencies: versionFixture.dependencies,
    readme: `# ${fixture.name}@${version}\n\nDeterministic npmx visual parity fixture.\n`,
    readmeFilename: 'README.md',
    dist: {
      shasum: `vize-e2e-${fixture.name}-${version}`,
      tarball: `${NPM_REGISTRY}/${encodePackageNameForTarball(fixture.name)}/-/${tarballName(
        fixture.name,
        version,
      )}`,
      integrity: versionFixture.integrity,
      fileCount: versionFixture.fileCount,
      signatures: [],
      unpackedSize: versionFixture.unpackedSize,
    },
  }
}

function isNpmRegistryUrl(url: URL): boolean {
  return url.origin === NPM_REGISTRY
}

function resolveRegistryPath(
  pathname: string,
): { packageName: string; versionSpecifier?: string } | null {
  const decoded = decodePathname(pathname)
  if (decoded === null) return null

  const segments = decoded.split('/').filter(Boolean)
  if (segments.length === 0 || segments[0]?.startsWith('-')) return null

  const packageSegments = segments[0]?.startsWith('@') ? segments.slice(0, 2) : segments.slice(0, 1)
  const packageName = packageSegments.join('/')
  const versionSpecifier = segments[packageSegments.length]
  return packageName ? { packageName, versionSpecifier } : null
}

function parseFastNpmMetaPath(
  pathname: string,
): { packageName: string; versionSpecifier?: string } | null {
  const decodedPathname = decodePathname(pathname)
  if (decodedPathname === null) return null

  const decoded = decodedPathname.replace(/^\/+/, '')
  if (!decoded) return null

  const versionSeparator = decoded.lastIndexOf('@')
  if (versionSeparator > 0) {
    return {
      packageName: decoded.slice(0, versionSeparator),
      versionSpecifier: decoded.slice(versionSeparator + 1),
    }
  }

  return { packageName: decoded }
}

function toUrl(request: string, baseURL: string): URL | null {
  try {
    return new URL(request, baseURL)
  } catch {
    return null
  }
}

function decodePathname(pathname: string): string | null {
  try {
    return decodeURIComponent(pathname)
  } catch {
    return null
  }
}

function encodePackageNameForTarball(name: string): string {
  return name.startsWith('@') ? name.replace('/', '%2F') : name
}

function tarballName(name: string, version: string): string {
  const basename = name.split('/').at(-1) ?? name
  return `${basename}-${version}.tgz`
}
