# Gauche tooling

Vize uses Gauche for repository automation whose main job is to coordinate
processes, files, and CI state. The Nix development shell and flake checks pin
the interpreter through `flake.lock`.

Run a script from the repository root so the shared modules are available:

```console
gosh -I tools/scheme path/to/script.scm
```

## Design rules

- Pass a program and its arguments separately through `vize.process`. Do not
  build a shell command string.
- Prefer one Gauche program for a complete workflow step. Repeated interpreter
  startup and state transfer between small scripts make the result slower and
  harder to follow.
- Keep parsing, validation, and decisions in Scheme. A YAML step should select
  inputs and invoke the program, not contain the implementation.
- Preserve exact stdout and exit-status behavior when replacing an existing
  script. Add a parity test before changing its caller.
- Leave third-party reproduction fixtures in their original language when the
  shell itself is part of the reported behavior.

Typed compiler, type-checker, linter, LSP, and formatter domain logic stays in
Rust or TypeScript unless a measured migration improves its boundary. Gauche is
the default for command orchestration and filesystem glue, not a reason to
discard useful static types.

## Runtime qualification

`tools/scheme/tests/process.scm` covers direct argument passing, output capture,
and nonzero exits. It also protects a macOS/Nix edge case: an infinite file
descriptor soft limit makes Gauche 0.9.15 process redirection fail while wiring
the child. `vize.process` gives only the Gauche process and its children the
ordinary finite Darwin ceiling before creating pipes.
