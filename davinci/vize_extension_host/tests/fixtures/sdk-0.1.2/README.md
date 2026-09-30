# Frozen SDK contract 0.1.2

`src/` and `wit/` are exact bytes from Vize commit
`2248de341f744f7d01f17b10a9432b40a826ab8b`. `source-sha256.json`
pins every archived file. The standalone manifest expands that revision's
workspace fields and dependency versions so this snapshot builds independently.

The legacy hello guest uses this SDK's original default export macro and page
writers. The legacy expression guest uses this SDK's runtime and archived WIT.
`wit_legacy` builds actual components, verifies their 0.1.2 imports/exports, and
exchanges exact committed pages with the current host in both execution modes.
