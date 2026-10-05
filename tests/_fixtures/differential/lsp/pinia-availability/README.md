# Pinia package availability regression

The two complete original files are retained byte-exact from issue #7827 and
pinned in `references.json`. The whole warning and fixed source are authored from
the existing public rule text and complete LSP diagnostic serializer, without
capturing current output. The real source-built RPC test first checks the absent
Pinia report, then creates the actual ancestor package identity, checks the
complete warning, applies `storeToRefs`, restores the original destructure, and
removes Pinia again. It also checks all original CLI preset results and input
conservation. No package installation, mocking of LSP transport, new parse stage,
old history manifest change or native/history admission is involved.
