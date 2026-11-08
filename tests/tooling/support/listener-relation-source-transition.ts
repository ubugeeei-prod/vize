import assert from "node:assert/strict";
import { createHash } from "node:crypto";
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
  current = emitMemberInferEditsBefore(readRepository, path, current);
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
