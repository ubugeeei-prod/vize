# Recover the pinned Rust script download on finite HTTP 5xx

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Qualification: source/process controls and hosted acceptance are distinct;
final-source Actions and protected delivery remain pending.

The original security job 114811979632 and title job 114812167307 of
[#8771](https://github.com/ubugeeei-prod/vize/pull/8771) failed in setup before
their tests. Both retained curl exit 22 / HTTP 504 for the original 0.34.0
Linux x64 URI. A later failed-only infrastructure retry passed; that is not
a durable repair or an explanation of the original failure. A separate
protected #8772 prefix also reported the same URI returning HTTP 500.

Decision: retain the original primary URI, curl flags, error stderr and exit
status. Only curl exit 22 together with exactly HTTP 500, 502, 503 or 504
permits one anonymous GET of the reviewed fixed API asset. No retries, loops,
cache trust, latest-version lookup, Cargo recovery or changed deadline are added.
A private suffix preserves the complete HTTP field across shell command
substitution so trailing newline cannot become an admitted status.

The [official release](https://github.com/fornwall/rust-script/releases/tag/0.34.0)
is release ID 122699694, with Linux GNU asset ID 127872817,
`rust-script-x86_64-unknown-linux-gnu.tar.gz`, size 1,255,442 bytes. The primary
URI remains
`https://github.com/fornwall/rust-script/releases/download/0.34.0/rust-script-x86_64-unknown-linux-gnu.tar.gz`.
The sole alternative is
`https://api.github.com/repos/fornwall/rust-script/releases/assets/127872817`,
with `Accept: application/octet-stream` and
`X-GitHub-Api-Version: 2022-11-28`. The
[official API contract](https://docs.github.com/en/rest/releases/assets#get-a-release-asset)
supports binary HTTP 200 or a redirect; the original HTTPS/TLS flags remain.

Independent full read-only GETs of both routes returned the same 1,255,442
bytes with archive SHA256
`0700ecfe6c768c007f3e5a7919f4fedbf0a6727d275a196791f134a1742d4d13`.
The archive contains exactly one regular `rust-script` member, mode 0755,
3,164,616 bytes, SHA256
`4a4e9ba691f47c2f51445bfbf03f9ea6c6323a5e01f3bc1630e41f17124d702a`.
Static ELF64/x86_64 inspection is not native execution or installed acceptance.

The read-only release metadata reported `immutable: false` and asset
`digest: null`. Its lightweight tag currently targets
`114d6e93cffe8d24f6915f4fdb4fb7b08568fd3e`. The fixed whole archive SHA,
not mutable metadata or the tag, remains the admission authority. Both routes
can reach `release-assets.githubusercontent.com`; this alternative does not
prove independent infrastructure or identify whether GitHub, CDN, proxy or
runner transport caused the observed 5xx.

The existing extractor's digest, complete membership/type checks and fresh
destination remain exact. The action still installs mode 0755, updates its
original PATH, checks 0.34.0, and cleans only its own temporary directory.
Windows linker setup, other-platform locked Cargo recipe and the already
matching runner fast path are unchanged. API JSON, partial/changed bytes,
unknown members, wrong version and failed transport remain fatal.

The original three installer laws and seven prebuilt laws remain. Only the
prebuilt positive curl argument vector gains its HTTP write-out argument.
New whole-action subprocess/file controls retain the original whole action
SHA256 `57c321c3ea4604a871184480fe1fd438cf297cd72f787a7563cd3a1a20772d33`
by exact reverse composition and show its original four 5xx refusals.
They cover all admitted statuses, primary success, authentication/404/429,
network/TLS/timeout/malformed status, failed API, full SHA/body/membership,
version, exactly one alternative and owned cleanup. Benign authored shell
runners and a child extractor fixture digest provide source/process proof;
the actual production digest argument remains asserted. No downloaded Linux
binary is executed on macOS. Final-source real Linux Actions must qualify the
original runner version and existing Rust script consumers before delivery.
