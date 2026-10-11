# Reuse the official pinned Rust script runner

Qualification: local controls pass; real Linux Actions are pending.
Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The retired merge-group Check [38135879923](https://github.com/ubugeeei-prod/vize/actions/runs/38135879923)
used source `872957a02d953bf4afe42f5c3e708f288ce43763`. Six jobs separately ran
`cargo install rust-script --version 0.34.0 --locked --force`. Their actual
Cargo compile times were 13.88, 13.69, 15.07, 16.47, 13.43 and 13.21 seconds:
85.75 seconds of aggregate command wall. Whole setup steps summed 126 seconds.
These are existing costs, not CPU measurements or post-change savings. The
Check itself passed; that candidate remained failed because Musea.

Decision: use the official 0.34.0 Linux x64 runner in the existing install
step. An already matching runner still takes the original fast path. Other
platforms still execute the original locked Cargo recipe. Installed-version
validation and its fatal outcome follow the same original PATH setup.

The [official release](https://github.com/fornwall/rust-script/releases/tag/0.34.0)
asset `127872817`, `rust-script-x86_64-unknown-linux-gnu.tar.gz`, is 1,255,442
bytes. Its full archive SHA256 is
`0700ecfe6c768c007f3e5a7919f4fedbf0a6727d275a196791f134a1742d4d13`.
The archive contains one regular `rust-script` file. The action passes that
fixed digest to a small extractor, which verifies it before any tar operation,
checks complete membership and file type, and extracts into a fresh temporary
directory. Aliases, extra entries, duplicate entries and traversal refuse.
The action installs mode 0755 and removes only its own temporary directory.

Static inspection found ELF64 x86_64, interpreter
`/lib64/ld-linux-x86-64.so.2` and GLIBC version strings through 2.34. The
official tag has empty default features and a release LTO profile. This is
static source/artifact evidence, not an ABI or Linux runtime success claim.

Local controls use benign authored shell runners, real tar and real SHA256.
The positive Mac fixture adapts only the child extractor's digest, and checks
that the actual action supplied the fixed production pin. A separate whole
action control retains that pin and refuses changed bytes before tar or exec.
Original installer controls remain intact. Version, download, malformed
archive, link, membership, platform and cleanup outcomes are covered.
All three original installer laws and seven new laws pass locally; the pinned
producer law rejects the original Cargo-only action. The existing tooling
planner selects both installer test modules once without a new stage.

No Linux binary is executed on macOS. Real Linux Actions must execute the
official runner's exact version and the existing Rust script commands before
this change is qualified. All original product recipes, checks, source
receipts, queue gates, timers and parallelism remain in place. The observed
85.75 seconds is an opportunity; download/validation costs and actual wall
benefit still require the genuine final-source run.
