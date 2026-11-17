/** The content-mapper producer before Pug templates got the echo directive table (#13284). */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const SUCCESSORS = [
  [
    `    // Echo ownership is read on the analyzed text (see \`directive_table\`).
    let analyzed_mappings = authored.is_derived().then(|| mappings.clone());
    authored.remap_mappings(&mut mappings);
`,
    `    authored.remap_mappings(&mut mappings);
`,
  ],
  [
    `    let directives = content_mapper_diagnostic_directives(
        &authored,
        &descriptor,
        &code,
        &spans,
        &diagnostic_echoes,
        &diagnostic_primaries,
        generic_copies.as_ref(),
        analyzed_mappings.as_deref().unwrap_or(&mappings),
        &mapping_meta,
        &callback_primaries,
        loose_unknown_props,
    );
    Ok(ContentMapperTransform {
        extension: CONTENT_MAPPER_VIRTUAL_EXTENSION,
        diagnostic_directives: with_owned_input_echoes(directives, &diagnostic_echoes),
`,
    `    Ok(ContentMapperTransform {
        extension: CONTENT_MAPPER_VIRTUAL_EXTENSION,
        // Directive comments are read from HTML templates and SFC blocks only.
        diagnostic_directives: with_owned_input_echoes(
            (!authored.is_derived())
                .then(|| {
                    content_mapper_diagnostic_directives(
                        content,
                        &code,
                        &descriptor,
                        &spans,
                        &diagnostic_echoes,
                        &diagnostic_primaries,
                        generic_copies.as_ref(),
                        &mappings,
                        &mapping_meta,
                        &callback_primaries,
                        loose_unknown_props,
                    )
                })
                .flatten(),
            &diagnostic_echoes,
        ),
`,
  ],
] as const;

/** `producer` with the derived-template echo table removed again. */
export function beforePugEchoTable(producer: string): string {
  let predecessor = producer;
  for (const [successor, restored] of SUCCESSORS) {
    assert.equal(
      predecessor.split(successor).length,
      2,
      "finite Pug echo-table producer successor",
    );
    predecessor = predecessor.replace(successor, restored);
  }
  assert.equal(
    createHash("sha256").update(predecessor).digest("hex"),
    "965d4f517cc073bc176ffed0653c3ed439854351bf6083794e35dadefb2436ed",
    "finite Pug echo-table producer successor",
  );
  return predecessor;
}
