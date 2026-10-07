# Reactive and recursive rename intersection

These complete authored inputs and repaired texts intersect #7996 reactive destructure with #7994 self-recursive same-name shorthand. They were authored and peer-reviewed before the first native query; they do not establish a product failure.

Public rename keeps the local `label` binding, changes the public type and destructure property, and expands Self/Parent public arguments to `:heading="label"`. Local rename keeps the public type and both public keys, expands the local binding and Self value to `heading`, and updates the explicit DOM value and interpolation. Parent's independent local stays unchanged in both routes.

The actual CLI test runs eight complete stdio sessions: four origins × LF/CRLF. It compares whole references, WorkspaceEdit, applied and disk bytes, and both versioned post-edit diagnostic arrays. The original report, original 24-session suite, and peer 40-session recursive suite retain their bytes and assertions.
