/** The #11675 producer adds only diagnostic-owned delimiter atoms. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { CHECK_MODE_PRODUCER_SUCCESSORS } from "./check-mode-successor.ts";
import { beforePugEchoTable } from "./pug-echo-table-successor.ts";
import { beforeVerbatimOffset } from "./verbatim-offset-successor.ts";

export function assertWrapperMappingProducer(producer: Uint8Array): void {
  const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
  assert.equal(
    digest(producer),
    "7ca582212b81c8a77956aef8b5213ef4be86d189f83e42297d156e34e7366e05",
    "unqualified #11675 wrapper mapping producer",
  );
  let historical = beforeVerbatimOffset(beforePugEchoTable(Buffer.from(producer).toString("utf8")));
  for (const [successor, predecessor] of [
    [
      `mod directive_table;
mod directives;
mod unmapped_ignores;
use directive_table::content_mapper_diagnostic_directives;
use directives::with_owned_input_echoes;
`,
      `mod directives;
mod unmapped_ignores;
use directives::{
    sfc_diagnostic_directives, with_component_prop_echoes, with_generic_echoes,
    with_loose_library_prop_fallthrough, with_owned_input_echoes,
};
`,
    ],
    [
      `                    content_mapper_diagnostic_directives(
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
`,
      `                    let directives = with_generic_echoes(
                        sfc_diagnostic_directives(
                            content,
                            &code,
                            &descriptor,
                            &spans,
                            &diagnostic_echoes,
                            &diagnostic_primaries,
                        ),
                        generic_copies.as_ref(),
                    );
                    let directives = with_component_prop_echoes(
                        directives,
                        &code,
                        &mappings,
                        &callback_primaries,
                    );
                    with_loose_library_prop_fallthrough(
                        directives,
                        content,
                        &code,
                        &mappings,
                        &mapping_meta,
                        loose_unknown_props,
                    )
`,
    ],
    [
      `    })
}
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
`,
      `    })
}
fn ranges_overlap(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
`,
    ],
  ] as const) {
    assert.equal(historical.split(successor).length, 2);
    historical = historical.replace(successor, predecessor);
  }
  assert.equal(
    digest(historical),
    "4faaf62203dc7d08d7b7e32c1495cdb3c1a6058b1b0cca0a9e0bbc6975f53a9c",
    "certified qualification-refactor wrapper successor",
  );
  for (const [successor, predecessor] of [
    [
      `use directives::{
    sfc_diagnostic_directives, with_component_prop_echoes, with_generic_echoes,
    with_loose_library_prop_fallthrough, with_owned_input_echoes,
};
`,
      `use directives::{
    sfc_diagnostic_directives, with_component_prop_echoes, with_generic_echoes,
    with_owned_input_echoes,
};
`,
    ],
    [
      `    let descriptor = prepare_script_blocks(descriptor);
    let loose_unknown_props =
        !super::vue_compiler_comments::apply(content, transform_options.virtual_ts_check_options())
            .check_unknown_props;
    let suppress_typescript_diagnostics =
`,
      `    let descriptor = prepare_script_blocks(descriptor);
    let suppress_typescript_diagnostics =
`,
    ],
    [
      `                    let directives = with_component_prop_echoes(
                        directives,
                        &code,
                        &mappings,
                        &callback_primaries,
                    );
                    with_loose_library_prop_fallthrough(
                        directives,
                        content,
                        &code,
                        &mappings,
                        &mapping_meta,
                        loose_unknown_props,
                    )
`,
      `                    with_component_prop_echoes(directives, &code, &mappings, &callback_primaries)
`,
    ],
  ] as const) {
    assert.equal(historical.split(successor).length, 2);
    historical = historical.replace(successor, predecessor);
  }
  assert.equal(
    digest(historical),
    "e70d9c79f81b29616c97f2ad334f081ef48b05be88141fefc6909aa6bfdae367",
    "certified loose-fallthrough wrapper successor",
  );
  // The verbatim-run successor returns the joined spans in place of `spans`.
  const verbatimRun = "    span_normalize::join_verbatim_runs(source, generated, spans)\n";
  assert.equal(historical.split(verbatimRun).length, 2);
  historical = historical.replace(verbatimRun, "    spans\n");
  assert.equal(
    digest(historical),
    "23000f014edd106b8ffe1cb1ebd9424f783a5cbd879cabf435f4bcfaffd51580",
    "certified verbatim-run wrapper successor",
  );
  for (const [successor, predecessor] of CHECK_MODE_PRODUCER_SUCCESSORS) {
    assert.equal(historical.split(successor).length, 2);
    historical = historical.replace(successor, predecessor);
  }
  assert.equal(
    digest(historical),
    "271dfc983bf196cd118b521e7ac401824b19c423d2cacfa32dcfec29bdec922b",
    "certified check-mode wrapper successor",
  );
  for (const [successor, expected, message] of [
    [
      "    span_features::retain_component_prop_hover_owner(generated, &mut spans);\n",
      "9c3faec15ca7c73c4a8a3f0fd1fcae7bb6f9d70c8e71a7046994d373e8b4e038",
      "certified component-hover owner wrapper successor",
    ],
    [
      "    span_features::retain_template_hover_owner(generated, &mut spans);\n",
      "6a060e4d0c5d16cd3071f76ca35558abcecde9fcd5458346d54287eacdef03fd",
      "certified template-hover owner wrapper successor",
    ],
    [
      "            template_use_navigation: true,\n",
      "d51c0acbed93e899ec5b634825bd3d7d2ffb2d47908f5bbc575e68feb909a937",
      "certified template-use projection wrapper successor",
    ],
  ] as const) {
    assert.equal(historical.split(successor).length, 2);
    historical = historical.replace(successor, "");
    assert.equal(digest(historical), expected, message);
  }
  const localOwnershipSuccessor = "            local_component_ownership_helpers: false,\n";
  assert.equal(historical.split(localOwnershipSuccessor).length, 2);
  historical = historical.replace(localOwnershipSuccessor, "");
  assert.equal(
    digest(historical),
    "f75096741294e3f99e5fedf6accad2daf49ee402c72af2bc684b20b0f4310e20",
    "certified local-ownership wrapper successor",
  );
  for (const addition of [
    "mod component_export;\n",
    "    component_export::insert_spans(&mut spans, mappings, metadata);\n",
  ]) {
    assert.equal(historical.split(addition).length, 2);
    historical = historical.replace(addition, "");
  }
  assert.equal(
    digest(historical),
    "25abafce233364f0b11f71202d32f87290ef7d231f939063938f8649e3448b67",
    "certified component-export wrapper successor",
  );
}
