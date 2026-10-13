# JSR distribution as an exact-version facade

Issue: [#8367](https://github.com/ubugeeei-prod/vize/issues/8367).

## Decision

Vize publishes one JSR package, `@vizejs/vize`. Each entry point re-exports an
npm package at the exact release version:

| JSR entry point       | npm package           |
| --------------------- | --------------------- |
| `@vizejs/vize`        | `vize`                |
| `@vizejs/vize/config` | `vize/config`         |
| `@vizejs/vize/native` | `@vizejs/native`      |
| `@vizejs/vize/vite`   | `@vizejs/vite-plugin` |

`tools/support/release/jsr/package.ts` generates the entry points into a
staging directory for the checked-out version, and the release publishes from
there. Only `jsr/vize/jsr.json`, its README and the channel switch are
committed.

## Why a facade

JSR hosts TypeScript and JavaScript modules. It does not host native addons
and does not run install scripts, so the compiler, linter and formatter
binaries cannot move to JSR. They stay in the existing per-platform npm
packages, and the JSR package depends on them through `npm:` specifiers. JSR
and npm users therefore run the same code and the same binaries, and there is
nothing to keep in sync besides the version number.

The CLI stays npm-only because a JSR package has no `bin` entry point. The
browser WASM package also stays on npm.

## Supported runtimes

Node.js 22 or newer, Deno 2 and Bun. The published npm packages already load
under all three (checked against 0.443.0 with Deno 2.9.7 and Bun 1.3.13), so
the earlier Node-only boundary is dropped.

## Checks

- Pull requests that touch the package run `deno publish --dry-run` on the
  generated package and import the staged entry points under Deno
  (`.github/workflows/jsr-package.yml`).
- `.github/workflows/release-jsr.yml` publishes through JSR's GitHub OIDC link
  and then installs the published version from JSR in fresh Node (Linux,
  macOS, Windows), Deno and Bun projects, importing every entry point.

## Channel switch

`jsr/vize/channel.json` holds `{"enabled": false}` until the `@vizejs` scope
and the `@vizejs/vize` package exist on JSR and the package is linked to this
repository. That setup needs the maintainer's JSR account. While the switch is
off, the release workflow skips publication. Once it is on, a failed
publication or consumer job fails the release.

## Supersedes

The draft in [#8388](https://github.com/ubugeeei-prod/vize/pull/8388) proposed
the same facade. This record replaces its Node-only boundary, its repository
variable and its frozen-source policy reader with the single committed switch
above.
