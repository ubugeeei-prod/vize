# v0.436.0 compatibility boundary

Decision paired with [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6050778335).

The unpublished 0.435.2 candidate failed the unchanged public Rust API gate:
`OperationNode::Key` and `IRNodeType::Key` add variants to exhaustive enums in
`vize_atelier_vapor`. These variants are necessary for keyed-fragment correctness;
Rust consumers with exhaustive matches must handle `Key` when upgrading.

[Cargo SemVer guidance](https://doc.rust-lang.org/cargo/reference/semver.html#enum-variant-new)
classifies this addition as a compatibility break. For 0.x releases, increasing
the second component establishes the new compatibility boundary. The supported
release is therefore **0.436.0**, using `vp run release minor -y --pin`.
The existing workflow derives SemVer classification from the candidate version;
its checks, baseline selection and failure policy remain unchanged.

Preserve the retired unpublished source #8243, integration #8244, H
`212692e21c618cc36e8281f041e2aa704c46eb0c`, run 37715578811 and complete
first-failure logs. Both PRs closed without merging, no v0.435.2 tag was created,
and only authenticated owned superseded workflow runs were cancelled.

The new cut uses actual signed main containing shipping-catalog fix #8237.
It must independently pass all five exact-head workflows, all twenty release
builds and full preflight, then deliver its version-only integration through the
protected queue. Authenticate actual delivery/catalog before tagging the immutable
source head, then verify all registries, assets, editor distribution, public
installed originals and Pages. Prior candidate greens grant no new-cut credit.
Ordinary merges continue; this separate documentation PR does not gate launch.

Publication remains pending until those checks succeed. This decision does not
claim complete n8n adoption, enforced ecosystem parity or completion of all P0s.
