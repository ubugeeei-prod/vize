# Private imports and redirected package diagnostics

`input.json` retains the five exact code-fence file bodies from the human report
https://github.com/ubugeeei-prod/vize/issues/11317. No lockfile or process exit status
was supplied. The reported package is Vize 0.632.0 (also 0.629.0), Vue 3.5.44 and
`typescript-native` aliased to npm:typescript@7.0.2 on Linux 6.18 WSL2, Node 24.22.1
and npm 11.19.1. Source Actions borrow the real existing test providers and source
CLI; that is separate from untouched public/WSL2 acceptance.

The compile-included native CLI test consumes all five original bodies, the three
reported control changes, a Vue importer, and repair/restoration in the same root.
Every full process result and whole JSON expectation is retained before semantic
assertions. The manifest row registers authored input only; it claims no native
admission or diagnostic pack. The earlier importing-file input remains separate.
