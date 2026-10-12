# Retirement runtime dependencies

Decision for [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The official retirement of unpublished v0.447.0 stopped during pinned MoonBit
setup, before any retirement guard or archive command ran. The complete
[original failure and source](../../../tests/_fixtures/release/retire-moon-447/manifest.json)
remain retained; HTTP403 alone does not establish the asset failure cause.
The original cut and its failed release remain immutable.

The hosted retire operation now calls the existing Rust facade with the same
source PR, H, tag, Release run and original operator run identities. MoonBit
setup and registry refresh run for start and resume. All operations still
install the frozen workspace graph. The existing local VP/Moon public
frontend remains available.

The Moon frontend previously refreshed its task registry before lowering these
same retirement arguments. That registry is not read by the Rust retirement
guards, Node absence helpers or archive collector. Retirement still keeps
Rust/Node/Git/GitHub setup, credentials and maintainer authority, immutable
cut checks, original terminal run checks, complete publication/channel
absence, archive custody and limits, concurrency and deadlines. Publication
and compiler qualification retain their existing setup and gates.

The original three workflow laws retain their authority and refusal corpus.
The actual parsed dependency shell now runs against argv capture models for
start, resume and retire, including frozen-install failure in each operation.
Start and resume must install before Moon registry refresh; retirement must
install without invoking Moon. The captured hosted retire command must reach
the exact existing six-argument Rust route. These models do not execute a
retirement or qualify public publication. The retained original dependency
shell is an executed counterexample: it invokes Moon during retirement and
fails the new dependency boundary. All three original workflow laws pass
locally, including the actual Rust starter's frozen-cut refusal controls;
scoped source lint reports no warnings.

Fresh source Actions, the full protected suites and actual merge are required
before the sole release operator can attempt the unchanged retirement.
Unpublished v0.447.0 is not retired by this source change; the last accepted
public version remains v0.443.0.
