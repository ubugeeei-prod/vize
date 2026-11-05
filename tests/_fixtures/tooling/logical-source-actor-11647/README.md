# Sealed logical source actor

`after.json.gz` contains the eight production files from
`7508c93edbf6cffed1ef081b9de73d7142af57de`, encoded as canonical Base64 JSON.
Its SHA-256 is `d3b4b96a46ca33364d0e32cea69ddeb96394c3f0492917374d324349b16e2eb5`.

Every decoded path, byte length and source hash must match the existing
`logical-workspace-overlay-revision-11145/source-authority.json` production
authority. These are historical test inputs, with no native qualification claim.
The source revision belongs to a separate stack and is not guaranteed to exist
in a fresh clone of the current branch.

Only the historical production reader consumes these images. Current observer
and tooling sources, explicit source-root overrides, and mutation controls keep
their existing readers. No historical authority, registration or source image
has been changed.
