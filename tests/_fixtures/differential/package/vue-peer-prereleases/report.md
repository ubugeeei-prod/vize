## Summary

`vize` declares `"vue": ">=2.6.0"` as an optional peer. Under semver, a range without a prerelease comparator does not match prerelease versions, so Vue 3.6 betas / RCs (the line that ships Vapor mode) are outside the range. npm treats this as a conflict and refuses to install `vize` next to `vue@rc` unless `--legacy-peer-deps` / `--force` is passed.

## Environment

- `vize` 0.432.0 (npm)
- `vue` 3.6.0-rc.10
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
npm init -y
npm install vue@3.6.0-rc.10 vize@0.432.0
```

## Actual

```text
npm error code ERESOLVE
npm error ERESOLVE could not resolve
npm error
npm error While resolving: vize@0.432.0
npm error Found: vue@3.6.0-rc.10
npm error
npm error Could not resolve dependency:
npm error peerOptional vue@">=2.6.0" from vize@0.432.0
npm error
npm error Conflicting peer dependency: vue@3.5.43
```

## Expected

`vize` installs next to Vue 3.6 prereleases without flags. For example, a range such as `">=2.6.0 || >=3.6.0-0"` keeps the current floor and also accepts the 3.6 prereleases:

```sh
npx semver -r ">=2.6.0 || >=3.6.0-0" 3.6.0-rc.10 3.6.0-beta.17 3.5.43 2.7.16 3.6.0
# 2.7.16, 3.5.43, 3.6.0-beta.17, 3.6.0-rc.10, 3.6.0
```

Related: #2142 (Vue peer failures in mixed Vue 2 / Vue 3 workspaces), which is a different case.

