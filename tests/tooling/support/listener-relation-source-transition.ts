import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { beforeSharedKebabCaseSource } from "../../../tools/support/compat/content-mapper/shared-kebab-case-transition.ts";
import { beforeSharedTemplateRefTargetsSource } from "../../../tools/support/compat/content-mapper/shared-template-ref-targets-transition.ts";
import { emitMemberInferEditsBefore, readRepository } from "./emit-member-infer-source.ts";

type Identity = { bytes: number; sha256: string };
type Edit = { before: string; after: string };
type Transition = { before: Identity; after: Identity; edits: Edit[] };

const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: hash(text),
});

const publicExport = `
import type * as __vizeListenerTypes from "./virtual-ts-listeners.js" with {
  "resolution-mode": "import",
};
export type $ListenerArgs<
  __P,
  __K extends string | number | symbol,
  __Raw extends string | number | symbol = never,
> = __vizeListenerTypes.$ListenerArgs<__P, __K, __Raw>;
`;

const cliDependencyOwnershipTransition: Transition = {
  before: {
    bytes: 4_273,
    sha256: "2e430bc9bbb14cbd07ae076459bf0a932d1558bb9a698b67527b4086d8988e6f",
  },
  after: {
    bytes: 4_362,
    sha256: "11abb0e379b919dcff38cc4aed99e1a8934aab4a55d9000115cf54dc1608f5c9",
  },
  edits: [
    { before: "", after: '    "json-schema-to-typescript": "catalog:repo-tooling",\n' },
    { before: "", after: '    "oxfmt": "catalog:linting",\n' },
  ],
};

const propsFallbackHeaderBefore =
  "declare namespace JSX { interface IntrinsicAttributes { class?: unknown; style?: unknown; } }\n" +
  "// vue-tsc generic library components accept unknown attrs through this\n" +
  "// fallback, which vue-tsc itself declares unless `checkUnknownProps` is set.\n" +
  "declare var __VLS_PROPS_FALLBACK: Record<string, unknown>;\n\n" +
  "// Shared type helpers used by generated virtual modules\n";
const propsFallbackHeaderAfter =
  "declare namespace JSX { interface IntrinsicAttributes { class?: unknown; style?: unknown; } }\n";
const propsFallbackManifestBeforeFiles =
  '    "generated/content-mapper-types.d.ts",\n' +
  '    "virtual-ts-checker.d.ts",\n' +
  '    "virtual-ts-emit-overloads.d.ts",\n' +
  '    "virtual-ts-globals.d.ts",\n' +
  '    "virtual-ts-listeners.d.ts",\n' +
  '    "virtual-ts-patterns.d.ts",\n';
const propsFallbackManifestAfterFiles = propsFallbackManifestBeforeFiles.replace(
  '    "virtual-ts-checker.d.ts",\n',
  '    "virtual-ts-checker.d.ts",\n    "virtual-ts-checker-strict.d.ts",\n',
);
const propsFallbackManifestBeforeExport =
  '      "types": "./virtual-ts-checker.d.ts"\n' +
  "    },\n" +
  '    "./content-mapper-helpers": {\n' +
  '      "import": {\n';
const propsFallbackManifestAfterExport =
  '      "types": "./virtual-ts-checker.d.ts"\n' +
  "    },\n" +
  '    "./virtual-ts-checker-strict": {\n' +
  '      "types": "./virtual-ts-checker-strict.d.ts"\n' +
  "    },\n" +
  '    "./content-mapper-helpers": {\n' +
  '      "import": {\n';
const propsFallbackPreambleBefore = [
  "",
  "pub const SHARED_PREAMBLE_DTS: &str = concat!(",
  '    include_str!("declarations/shared-header.d.ts.txt"),',
  "    vue_type_helpers_text!(),",
  '    include_str!("declarations/shared-template-refs.d.ts.txt"),',
  '    include_str!("declarations/shared-slot-resolver.d.ts.txt"),',
  '    include_str!("declarations/shared-slot-payload.d.ts.txt"),',
  "    emit_overload_helpers_text!(),",
  '    include_str!("declarations/shared-emit-props.d.ts.txt"),',
  '    include_str!("declarations/shared-emit-inference.d.ts.txt"),',
  '    include_str!("declarations/shared-listener-args.d.ts.txt"),',
  '    include_str!("declarations/component-checks.d.ts.txt"),',
  '    include_str!("declarations/shared-macros.d.ts.txt"),',
  '    include_str!("pattern_matching.d.ts")',
  ");",
  "",
  "",
].join("\n");
const propsFallbackPreambleAfter = [
  "",
  "/// The program-wide checker preamble. vue-tsc declares its props fallback only",
  "/// for files that leave `checkUnknownProps` off, so the fallback is a segment.",
  "macro_rules! shared_preamble_text {",
  "    ($props_fallback:expr) => {",
  "        concat!(",
  '            include_str!("declarations/shared-header.d.ts.txt"),',
  "            $props_fallback,",
  '            include_str!("declarations/shared-helpers-banner.d.ts.txt"),',
  "            vue_type_helpers_text!(),",
  '            include_str!("declarations/shared-template-refs.d.ts.txt"),',
  '            include_str!("declarations/shared-slot-resolver.d.ts.txt"),',
  '            include_str!("declarations/shared-slot-payload.d.ts.txt"),',
  "            emit_overload_helpers_text!(),",
  '            include_str!("declarations/shared-emit-props.d.ts.txt"),',
  '            include_str!("declarations/shared-emit-inference.d.ts.txt"),',
  '            include_str!("declarations/shared-listener-args.d.ts.txt"),',
  '            include_str!("declarations/component-checks.d.ts.txt"),',
  '            include_str!("declarations/shared-macros.d.ts.txt"),',
  '            include_str!("pattern_matching.d.ts")',
  "        )",
  "    };",
  "}",
  "",
  "/// Preamble of a program with at least one file that leaves `checkUnknownProps` off.",
  "pub const SHARED_PREAMBLE_DTS: &str =",
  '    shared_preamble_text!(include_str!("declarations/props-fallback.d.ts.txt"));',
  "/// Preamble of a program whose every file checks unknown props (#11628).",
  'pub const SHARED_PREAMBLE_STRICT_DTS: &str = shared_preamble_text!("");',
  "",
  "",
].join("\n");
const propsFallbackPreambleTransition: Transition = {
  before: {
    bytes: 5_052,
    sha256: "15ad52d5dbc3166ba6fb5d7386515b6929064862ac530795c6a56525b539076e",
  },
  after: {
    bytes: 5_816,
    sha256: "a8ef812b81fda63ef7d2f9104cccffaa5330df2ab5ea4e49360410536f0e4464",
  },
  edits: [{ before: propsFallbackPreambleBefore, after: propsFallbackPreambleAfter }],
};
const propsFallbackSourceTransitions = new Map<string, Transition>([
  [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-header.d.ts.txt",
    {
      before: {
        bytes: 967,
        sha256: "1c0fc02eb692116e0b06156b95cfc5e7d1b6e6d023d24bb21ab09c993937bbba",
      },
      after: {
        bytes: 700,
        sha256: "f08b5f2e153ff1de311598a9549b9b8a2b541067148a97f0b2f3211cf65413cf",
      },
      edits: [{ before: propsFallbackHeaderBefore, after: propsFallbackHeaderAfter }],
    },
  ],
  [
    "npm/cli/package.json",
    {
      before: {
        bytes: 4_362,
        sha256: "11abb0e379b919dcff38cc4aed99e1a8934aab4a55d9000115cf54dc1608f5c9",
      },
      after: {
        bytes: 4_494,
        sha256: "e4fa0c743fbc3fe1efec08f09ee72c38b2faa476e831f0233ea05efea63644a1",
      },
      edits: [
        { before: propsFallbackManifestBeforeFiles, after: propsFallbackManifestAfterFiles },
        { before: propsFallbackManifestBeforeExport, after: propsFallbackManifestAfterExport },
      ],
    },
  ],
  ["crates/vize_canon/src/virtual_ts/helpers/preamble.rs", propsFallbackPreambleTransition],
]);

const sourceTransitions = new Map<string, Transition>([
  [
    "crates/vize_canon/src/virtual_ts/helpers/preamble.rs",
    {
      before: {
        bytes: 4_988,
        sha256: "38c8e437524c3983a57d089febbf49ac4898ddfc1318672ded78608728c0dde6",
      },
      after: {
        bytes: 5_052,
        sha256: "15ad52d5dbc3166ba6fb5d7386515b6929064862ac530795c6a56525b539076e",
      },
      edits: [
        {
          before: "",
          after: '    include_str!("declarations/shared-listener-args.d.ts.txt"),\n',
        },
      ],
    },
  ],
  [
    "npm/cli/scripts/generate-content-mapper-types.ts",
    {
      before: {
        bytes: 18_554,
        sha256: "9df0e4ea2c0f466c3f717192102e3b6f14350540987d9dc3bc573111823a13c4",
      },
      after: {
        bytes: 18_736,
        sha256: "ffabac531fe547766e1e5913292b048c5478b831ad2bec8f84713b060c8987f5",
      },
      edits: [
        {
          before: "",
          after: 'import { listenerArgumentExports } from "./listener-argument-exports.ts";\n',
        },
        {
          before: "",
          after:
            "for (const [file, contents] of await listenerArgumentExports(root, outputs))\n  outputs.set(file, contents);\n",
        },
      ],
    },
  ],
  [
    "npm/cli/package.json",
    {
      before: {
        bytes: 4_240,
        sha256: "cb84f2c91b84e60fbec19e3745e4fad6b8e0a35f821f54ea9b9d6e5d8e3478d8",
      },
      after: {
        bytes: 4_273,
        sha256: "2e430bc9bbb14cbd07ae076459bf0a932d1558bb9a698b67527b4086d8988e6f",
      },
      edits: [{ before: "", after: '    "virtual-ts-listeners.d.ts",\n' }],
    },
  ],
  ...["ts", "mts", "cts"].map(
    (extension) =>
      [
        `npm/cli/virtual-ts.d.${extension}`,
        {
          before: {
            bytes: 15_065,
            sha256: "417b4ab1f380eaffeb62676def8762ed78fba126e606972f262503b8cda04432",
          },
          after: {
            bytes: 15_357,
            sha256: "d2afee78f759b85aa1b227caf4bd0b6f9fef54671e0c6e5f7c89f40c9b12cd6e",
          },
          edits: [{ before: "", after: publicExport }],
        },
      ] as [string, Transition],
  ),
  [
    "crates/vize_canon/src/virtual_ts/scope/component_events.rs",
    {
      before: {
        bytes: 22_127,
        sha256: "3175d7d98746e353a939040e3a80704f9e6d6aa9960b4d85be1bbadb1c51852d",
      },
      after: {
        bytes: 22_190,
        sha256: "a8a74895bb7d5b6a33392f50455cae826a4e5754e59cfc6cce6641876c8fe4e3",
      },
      edits: [
        { before: "", after: "mod listener_relations;\n" },
        { before: "", after: "    pub(super) share_listener_relations: bool,\n" },
        { before: "", after: "        share_listener_relations,\n" },
        {
          before: "",
          after:
            "    let mut listener_key = String::default();\n    push_ts_string_literal(&mut listener_key, on_handler.as_str());\n",
        },
        {
          before: "",
          after:
            "    let relations = listener_relations::ListenerRelations::new(\n        &component_aliases,\n        hoist_shared_preamble && share_listener_relations,\n    );\n",
        },
        {
          before:
            '    let missing_prop_args = if data.event_name.contains(\'-\') {\n        let mut raw_handler = String::from("on");\n        let mut chars = data.event_name.chars();\n        if let Some(first) = chars.next() {\n            raw_handler.extend(first.to_uppercase());\n        }\n        raw_handler.push_str(chars.as_str());\n        let mut raw_key = String::default();\n        push_ts_string_literal(&mut raw_key, &raw_handler);\n        cstr!("__P extends {{ {raw_key}?: (...args: infer __A) => any }} ? __A : unknown[]")\n    } else {\n        String::from("unknown[]")\n    };\n',
          after:
            '    let raw_listener_key = if data.event_name.contains(\'-\') {\n        let mut raw_handler = String::from("on");\n        let mut chars = data.event_name.chars();\n        if let Some(first) = chars.next() {\n            raw_handler.extend(first.to_uppercase());\n        }\n        raw_handler.push_str(chars.as_str());\n        let mut raw_key = String::default();\n        push_ts_string_literal(&mut raw_key, &raw_handler);\n        raw_key\n    } else {\n        String::from("never")\n    };\n',
        },
        { before: "", after: "        relations.emit(ts, indent);\n" },
        {
          before:
            '    let prop_listener_args = cstr!(\n        "__P extends {{ {prop_key}?: (...args: infer __A) => any }} ? __A : __P extends {{ {prop_key}?: infer __H }} ? [Extract<NonNullable<__H>, (...args: any) => any>] extends [never] ? {missing_prop_args} : Extract<NonNullable<__H>, (...args: any) => any> extends (...args: infer __A) => any ? __A : {missing_prop_args} : {missing_prop_args}"\n    );\n',
          after:
            '    let prop_listener_args = cstr!(\n        "{}<__P, {listener_key}, {raw_listener_key}>",\n        relations.args\n    );\n',
        },
      ],
    },
  ],
  [
    "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/integrated_source.rs",
    {
      before: {
        bytes: 20_362,
        sha256: "40ec1d06ab3d9e34442d45d2f6ab1270734537e33e19e6be04308e25277cfc24",
      },
      after: {
        bytes: 20_509,
        sha256: "ef9920da2fbb48f87459211ac6b7934bfa3fb18794162f90b3815d7d4adc1f0d",
      },
      edits: [
        {
          before: "",
          after: "    let current = &super::listener_relations::preamble_before(current);\n",
        },
        {
          before: "",
          after: "    let current = super::listener_relations::source_before(path, current);\n",
        },
      ],
    },
  ],
]);

const guardedInferenceTransition: Transition = {
  before: {
    bytes: 22_190,
    sha256: "a8a74895bb7d5b6a33392f50455cae826a4e5754e59cfc6cce6641876c8fe4e3",
  },
  after: {
    bytes: 22_600,
    sha256: "b4e95857da9131f19273c08925849109987b6bae8af8eac01d069484ba2860d1",
  },
  edits: [
    {
      before: "    pub(super) enclosing_vif_guard: Option<&'a str>,\n",
      after:
        "    /// Logical guard that keeps listener contracts scoped to their authored\n" +
        "    /// branch, even when the inference call needs its own physical wrapper.\n" +
        "    pub(super) enclosing_vif_guard: Option<&'a str>,\n" +
        "    /// Guard already wrapped around the emitted inference call. Only this may\n" +
        "    /// be subtracted from the call's own `v-if` guard.\n" +
        "    pub(super) inference_outer_vif_guard: Option<&'a str>,\n",
    },
    {
      before: "        enclosing_vif_guard,\n        event_inference_usages,\n",
      after:
        "        enclosing_vif_guard,\n        inference_outer_vif_guard,\n        event_inference_usages,\n",
    },
    {
      before: "            enclosing_vif_guard,\n            event_inference_usages,\n",
      after:
        "            enclosing_vif_guard: inference_outer_vif_guard,\n            event_inference_usages,\n",
    },
  ],
};

function inverse(current: string, transition: Transition, context: string): string {
  assert.deepEqual(identity(current), transition.after, context);
  for (const edit of transition.edits) {
    assert.ok(edit.after, context);
    assert.equal(current.split(edit.after).length, 2, context);
    current = current.replace(edit.after, edit.before);
  }
  assert.deepEqual(identity(current), transition.before, context);
  return current;
}

/** Restore only the authenticated pre-listener-relation source image. */
export function beforeSharedListenerRelationSource(path: string, current: string): string {
  current = beforeSharedTemplateRefTargetsSource(path, current);
  current = beforeSharedKebabCaseSource(path, current);
  current = emitMemberInferEditsBefore(readRepository, path, current);
  const propsFallbackTransition = propsFallbackSourceTransitions.get(path);
  if (
    propsFallbackTransition &&
    identity(current).sha256 === propsFallbackTransition.after.sha256
  ) {
    current = inverse(
      current,
      propsFallbackTransition,
      `selected emit gate producer: props fallback source ${path}`,
    );
  }
  if (
    path === "npm/cli/package.json" &&
    identity(current).sha256 === cliDependencyOwnershipTransition.after.sha256
  ) {
    current = inverse(
      current,
      cliDependencyOwnershipTransition,
      "selected emit gate producer: CLI dependency ownership",
    );
  }
  if (
    path === "crates/vize_canon/src/virtual_ts/scope/component_events.rs" &&
    identity(current).sha256 === guardedInferenceTransition.after.sha256
  ) {
    current = inverse(
      current,
      guardedInferenceTransition,
      "selected emit gate producer: guarded inference source",
    );
  }
  const transition = sourceTransitions.get(path);
  return transition
    ? inverse(current, transition, `selected emit gate producer: listener relation source ${path}`)
    : current;
}
