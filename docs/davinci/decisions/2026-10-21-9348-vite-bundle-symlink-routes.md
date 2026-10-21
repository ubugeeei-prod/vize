# Vite bundle symlink-route boundary

Issue: [#9348](https://github.com/ubugeeei-prod/vize/issues/9348).

The native configuration loader uses the selected project's public
`loadConfigFromFile(..., "bundle")` contract. Successful loads return the
selected config and canonical bundled dependencies. They do not return the
original lexical request route when a relative import passes through a symbolic
link. Editing a canonical target is therefore observable, but retargeting only
the link is not.

The editor guide now states the operational boundary: after changing only such
a link target, run **Vize: Restart Server**. Ordinary edits to the selected Vite
configuration or a reported imported helper continue to reload without a
restart. The real-loader regression retains both sides of the contract: each
fresh load follows the new link target, the canonical target remains in the
dependency set, and the unreported route is not presented as watched.

This decision does not manufacture provenance by parsing authored imports,
polling project files, replacing project Vite, evaluating the configuration
factory twice, or treating source maps as consumed-input evidence. Automatic
retarget observation remains blocked on an authentic resolver/raw-read
observation seam in the selected bundler. When that public contract exists,
the regression must change from documenting the boundary to requiring the
lexical route and the restart guidance can be removed.
