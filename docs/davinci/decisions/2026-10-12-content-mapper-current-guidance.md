# Current Content Mapper host guidance

Issues: [#8372](https://github.com/ubugeeei-prod/vize/issues/8372) and
[#8365](https://github.com/ubugeeei-prod/vize/issues/8365).

The complete English and Japanese Content Mapper guides still direct readers
to the archived `typescript-go` main branch, describe protocol v1, and say
declaration maps await the old #4860 issue. These statements conflict with
the actual host and mapper that Vize's existing conformance workflow tests.

The [archived staging repository](https://github.com/microsoft/typescript-go)
directs current development to `microsoft/TypeScript`. The
[round 2 change](https://github.com/microsoft/TypeScript/pull/63936), merged as
`d6c4afddb2c55f4a9dea7b59293a99a8fdea1799`, removed numbered protocol
negotiation and completed the
[transferred declaration-map request](https://github.com/microsoft/TypeScript/issues/63880).
Vize's actual conformance workflow already checks out this exact TypeScript
revision and builds `tsc/cmd/tsc` into the deliberately named `tsgo` executable.
The current mapper's real initialization tests reject legacy version echoing
and select UTF-8; these provider and workflow sources are unchanged here.

Historical full [Content Mapper conformance run 38107308203](https://github.com/ubugeeei-prod/vize/actions/runs/38107308203)
passed at Vize source `f3fea32a4d6590dc504fc428223cb3119cbfbd67`.
Its job `114375419822` records successful
`standard_tsgo_emits_authored_vue_declaration_maps` and
`standard_tsgo_lsp_uses_vue_declaration_maps_from_packed_consumer` execution.
This is evidence for the stated exact host and those original native cases,
not a newly released all-platform or arbitrary nightly-package receipt.

Correct both guides to describe that host and contract. Retain the current
production `vize check` path, trusted-workspace condition, explicit external
code opt-in, every option default and diagnostic, template directive scope,
project lifecycle, `.tsx` output and original destinations. Explain the
qualified `tsgo` command name without claiming that every published `tsc` or
old native-preview package exposes the API. The declaration-map request is
historical completed work; other host revisions require their own acceptance.

Review the complete Japanese body in natural Japanese and retain its six
original headings. All five original whole fences are byte-exact in both
locales; their JSON-vector SHA-256 is
`8940b963ebe24b35e139a63098e85a348de702114d24b7d243b4497cf01ffcea`.
The existing translation retention test now exercises real collection of this
complete reviewed document. Its original generated input fails before the
change; the candidate retains the complete body and all runnable recipes.

The page is in the existing analysis navigation group, not the More fallback;
do not alter the unrelated More coverage classifications. No compiler,
provider, package, editor, workflow, conformance oracle or upstream file is
changed. This slice requires exact-source Actions, the original full Docs,
protected actual merge and native public desktop/mobile reading acceptance.
The whole Japanese and More guides issues remain open.

The first source Actions run rejects naming the retired native-preview
package in published guidance, even as a negative example. Preserve the
existing package-runtime guard and describe only the actual pinned native
entrypoint and binary name in both guides. All five whole recipes and
destinations remain exact; fresh source qualification is required.
