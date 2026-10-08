# Preserve quoted event definitions in Content Mapper protocol spans

Tracker: [#4075](https://github.com/ubugeeei-prod/vize/issues/4075).

## Observed regression

The scheduled [Content Mapper Conformance run
37769763871](https://github.com/ubugeeei-prod/vize/actions/runs/37769763871)
tested main `8816b0d594ce9a5322ab33f60be5ccb6839fa774` with the unchanged
TypeScript provider `d6c4afddb2c55f4a9dea7b59293a99a8fdea1799`.
`standard_tsgo_lsp_maps_event_symbol_navigation` received `[]` for the
call-signature event `submit`, including its existing 60 readiness retries.
The earlier identical fixture, test, workflow and provider passed at
`4b69ba1dae170b431d566f218547b9bcd6e27cb0` in run `37628244978`.

The original request is `App.vue` at zero-based UTF-16 position `40:23`.
The declaration is the complete literal `"submit"` in `CallSignatureChild.vue`,
at `2:10–18`, source bytes `[49,57)`. The original stock test checks the
target URI and start position; it has no whole definition packet golden.
Its rename assertions require both the parent `[40:23,40:29)` and child
interior `[2:11,2:17)` edits. These expectations and input files remain intact.

## Cause and bounded correction

The shared authored-event producer intentionally retains a whole generated
quoted literal mapped to the authored name interior, followed by the exact
interior mapping. Vize's native definition and rename consumers need both
endpoints. Its source and the native whole-packet history remain unchanged.

Content Mapper's ordinary `candidate()` narrows a containing generated string
to the original substring. Both entries therefore become the same interior
Verbatim candidate. Protocol normalization drops the duplicate, leaving the
generated quotes uncovered. The pinned TypeScript definition consumer maps
the entire string-literal name and rejects targets without single-segment
fidelity, so it returns an empty location array.

Correct this only in the private protocol candidate producer. Examine adjacent
whole/interior entries in one pass. Accept a pair only when:

- Both entries have no sub-spans and the same nonempty original range.
- The companion generated range is exactly the whole range's interior.
- The whole generated text is enclosed in matching single or double quotes;
  its unescaped interior equals the retained source bytes.
- Expanding the original range by one byte on each side yields the identical
  full quoted text, with checked boundaries.
- Both ordinary candidates and the replacement have equal computed protocol
  feature metadata. No feature is enabled by this correction.

Produce one exact full quoted Verbatim candidate for this protocol consumer,
consuming only the verified adjacent pair. Unpaired, mismatched, escaped,
empty, sub-span or feature-mismatched entries use the unchanged ordinary path.
There is no search across other mappings, extra parse, pipeline stage, new
public field, approximate projection or global narrowing change.

## Qualification

Add a direct regression using the original `CallSignatureChild.vue` fixture:
the whole generated literal must project to the complete original literal,
with its existing features and without generated overlap. Add independently
authored positive and refusal controls for the private pair boundary.

Preserve the scheduled baseline failure. Run the unchanged original
`cargo test -p vize --test content_mapper_tsgo_lsp_event_forms -- --nocapture`
through the existing Content Mapper workflow with the same pinned provider on
the exact final source. Existing hosted source and protected suites still
apply, including all original #4075/native history inputs and whole packets.

The baseline log records only the returned `[]`; it has no transform artifact
or absolute generated offset. Do not invent that offset or claim a previous
complete stock response was captured. The local generated geometry is the
whole `[G,G+8)` and interior `[G+1,G+7)`; successful hosted qualification must
establish the actual source behavior.

This is a repair of a verified Content Mapper regression. It does not complete
broad #4075, replace native rename ownership or establish public publication.
