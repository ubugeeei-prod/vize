# Type-checking performance

Use this guide when the type check is slow or uses too much memory. On TypeScript 7.1 or newer,
the recommended type check is the [content mapper](./content-mapper.md), run by `tsc` or
[tsrs](https://github.com/oxc-project/tsrs). `vize check` remains for projects that cannot move to
TypeScript 7.1. Start with the defaults and a profile, then change one setting at a time. For the
other options, see [type-checking with the CLI](./cli.md#check) and the
[Type Checking guide](./type-checking.md).

## Find where the time goes

### Content mapper

Add `--extendedDiagnostics` to the content mapper command:

```sh
tsrs --runExternalCode --noEmit -p tsconfig.json --extendedDiagnostics
```

TypeScript 7.1 accepts the same flag:

```sh
tsc --runExternalCode --noEmit -p tsconfig.json --extendedDiagnostics
```

The output lists the files checked, the type, symbol and instantiation counts, memory use, and
the time spent in each compiler phase.

### `vize check`

```sh
vpx vize check --profile
```

The last line splits the run into phases:

```diagnostic
✗ Type checked 1 files in 624.27ms (collect: 105.94ms, imports: 0.00ns, gen: 114.51ms, corsa: 339.05ms)
```

| Phase     | What it covers                                                      |
| --------- | ------------------------------------------------------------------- |
| `collect` | Reading `tsconfig.json` and finding the files to check              |
| `imports` | Following imports between components                                |
| `gen`     | Turning each `.vue` file into virtual TypeScript                    |
| `corsa`   | Running TypeScript's native compiler (Corsa) on the generated files |

The profile table and the virtual files are written under `node_modules/.vize`. When `corsa`
dominates, the notes at the end name the largest generated files; those are usually the
components to split or simplify.

## Report a slow project

A slow run is worth an [issue](https://github.com/ubugeeei-prod/vize/issues/new), even a rough one.
Report with the checker you actually use: the content mapper output for a TypeScript 7.1 or tsrs
project, and `vize check --profile` otherwise. Include:

- the command and its full `--extendedDiagnostics` or `--profile` output;
- the Vize version, and the TypeScript or tsrs version;
- the number of `.vue` files, and the OS, CPU core count and memory of the machine.

## Tune `vize check`

| Symptom                              | Try                                                                         |
| ------------------------------------ | --------------------------------------------------------------------------- |
| Uses too much memory                 | `--servers 1` runs a single TypeScript process                              |
| Large project, idle CPU cores        | Leave `--servers` unset; it is sized from CPU count and file count          |
| `corsa` dominates on one big project | `--checkers 4` adds checker workers inside each process (see below)         |
| Many short runs while editing        | Keep a [`check-server`](./type-checking.md#speed-on-large-projects) running |

```sh
vpx vize check --checkers 4 --servers 1
```

> [!WARNING]
> More than one checker worker per process can drop or change diagnostics, so the default is one.
> Compare the output with the default before you use `--checkers` in CI.

`--checkers` accepts a positive integer and overrides the `VIZE_CHECKERS` environment variable.
`--servers` (`1`–`32`) sets how many separate TypeScript processes run; automatic sizing accounts for
the checker workers in each process. Set the server count for every run with `vue.typeChecker.servers`.
