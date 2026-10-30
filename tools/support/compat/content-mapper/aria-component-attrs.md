# Component ARIA package acceptance (#11144)

`aria-component-attrs.ts` runs the original report, plain/bound ARIA variants,
no-props and camel-declared children, a native-element control, and declared-prop
valid → invalid → repaired cycles. CRLF with a Unicode prefix is included for the
original and the prop edit cycle. It checks all mapper stdout, stderr and exit
statuses, including exact authored UTF16 error positions. Every protocol response,
process result, input byte hash and package declaration hash is retained.
Each physical SDK first checks an independent handwritten TypeScript prop
assignment. Its exact `false` type error supplies the negative contract; the
Vue diagnostics must retain the authored attribute-name coordinates.

The 20 phases include two required-camel-ARIA repair cycles. They read the exact
normal Batch fixture files in `aria-propertykey-capture-11153`: the authored
`type PropertyKey = 0` case and its blank-line control. Both valid inputs must
retain the required `ariaLabel`; both invalid `size` values must retain the same
complete authored error. The original 14 phases remain unchanged.

The runner requires the actual `vize/virtual-ts` package import and the shipped
declaration export conditions. An embedded-helper fixture would miss this bug.
Source mode copies all shipped declaration files and the actual CLI manifest,
changing only its mapper command to the supplied source CLI. Installed mode uses
the installed package and its mapper command without modifications.
For a binary built in another clean checkout, pass `--source-package` with that
checkout's `npm/cli` path. Its Git revision must equal `--source-sha`; installed
mode refuses this override.

After the existing component-input workflow builds its source CLI and installs
the three physical SDKs, invoke once per SDK:

```sh
node tools/support/compat/content-mapper/aria-component-attrs.ts \
  --consumer "$RUNNER_TEMP/component-input-consumer" \
  --candidate-cli "$GITHUB_WORKSPACE/target/ci/vize" \
  --source-sha "$SOURCE_SHA" \
  --mapper-host "$mapper_host" \
  --mapper-version "$mapper_version" \
  --output "$RUNNER_TEMP/component-input-ownership/aria-package/$sdk"
```

Use the already authenticated SDK image and exact version from that workflow:
the image resolved by the installed TypeScript `lib/getExePath.js` for actual-20261024,
the beta consumer's native image for beta-3e923, and the built `component-input-d6c4/tsgo`
for d6c4. Obtain each complete version with `"$mapper_host" --version`.

For public release acceptance, install the target `vize`, `tsrs` and `vue`
versions in a fresh consumer, then omit both source arguments:

```sh
node tools/support/compat/content-mapper/aria-component-attrs.ts \
  --consumer /absolute/path/to/fresh-consumer \
  --mapper-host /absolute/path/to/fresh-consumer/node_modules/@tsrs/linux-x64/tsrs \
  --mapper-version 'the complete exact --version output without its final newline' \
  --output /absolute/path/to/new-capture-directory
```

A source run proves that source CLI with its shipped declaration files. Public
release acceptance requires a separate installed-mode run. The runner never
overwrites an existing capture and never changes the installed package.
