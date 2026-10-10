# Portable Musea snapshot ownership

Issue: [#8424](https://github.com/ubugeeei-prod/vize/issues/8424).

The snapshot collision preflight prevents a corrupt batch, but it does not
provide usable independent baselines. Conditional filename changes alone are
insufficient: a later subset capture or removal can reuse a different Art's
legacy PNG. This slice assigns persistent ownership before any screenshot
navigation or PNG write.

## Identity and naming

The static emitter adds `snapshotIdentityVersion: 1` and a
`snapshotIdentities` map from existing Art paths to normalized POSIX paths
relative to `MuseaOptions.projectRoot ?? Vite.root`. Existing Art paths,
preview IDs and preview URLs remain unchanged. Hosted capture requires that
versioned map and never infers the remote build root from the local checkout.
An old hosted gallery requires a rebuild. External Art includes require an
explicit common `projectRoot`; absolute machine paths are never hashed as
logical identities.

The capture identity is the versioned JSON tuple of relative Art path,
variant name, viewport name, width, height and device scale. Ordinary new
captures retain their existing PNG names. Whole-batch filename collisions
use `snapshot-<full SHA-256 of capture identity>.png`. Case-folded legacy
names are compared together for portability across filesystems. Long or
otherwise unsafe legacy filenames also use the bounded hash filename.

`identities.json` persists each filename's owner. The runner reserves the
whole batch atomically before capture and retains the exclusive
`identities.lock` through capture, approval and shutdown. Plan calls from
parallel workers are serialized; unchanged reservations are not rewritten.
Owners remain reserved after clean, so removal cannot transfer a baseline
to another Art. Corrupt indexes, unsupported identities, missing map entries,
duplicate captures and concurrent runs fail before PNG mutation.

## Migration and operations

An existing unowned, unambiguous legacy PNG requires explicit
`--adopt-legacy-snapshots` after reviewing its component/variant/viewport.
Ambiguous legacy PNGs cannot be adopted; new qualified baselines must be
captured and reviewed. The index and baseline PNGs must be versioned or
transferred together. Changing `projectRoot` changes relative identity and
requires a reviewed migration. No guessed legacy copies are created.

Approval accepts project-relative patterns such as `right/Button/*` and
refuses basename patterns that match multiple Arts, including when only one
currently fails. Pattern matching escapes literal regex characters in real
Art/variant names and distinguishes component-local `*` from path-crossing
`**`. Capture and clean share job enumeration, including
`variant.args.viewport`. CI failures set the exit code and allow runner
cleanup; an immediate `process.exit()` would leave the ownership lock behind.

If a process crashes, verify that no VRT process uses the directory before
removing the stale `identities.lock`. No automatic stale-lock takeover is
performed. Index writes use atomic rename; this does not promise filesystem
crash durability through `fsync`.

## Evidence and remaining gates

Two persisted genuine `left/Button.art.vue` and `right/Button.art.vue`
sources remain byte-for-byte in the tooling fixture directory. Native Art
parsing belongs to the collision preflight's package contract; this browser
contract deliberately renders their explicit styles through real Vue preview
modules, production static HTML/manifest emission and an HTTP host. It does
not claim native component compilation.

Local Chromium exercised 16 independent baselines, 16 repeated matches,
16 matches from a rebuilt different machine root, two isolated right-Art
diffs with CI exit 1, refusal of ambiguous approval, targeted right approval,
14 matches after left removal, and clean preserving the full ownership
index. Original source paths are removed before hosted capture. Browser
reports, PNGs, indexes and HTTP requests are retained as Actions artifacts.
Source controls cover legacy adoption, case/delimiter/empty/Unicode names,
viewport dimensions/scale, corrupt indexes, locks, relocation and override
cleanup. Fresh exact-head Actions, protected Stack delivery and publication
remain required; collision refusal alone grants no VRT completion credit.
