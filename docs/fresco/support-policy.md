# Fresco stability and extension support

Fresco is incubating. This policy defines what users can rely on now and how incompatible changes
are communicated while the packages remain below 1.0.

## Versioning guarantees

- Patch releases do not intentionally break documented public TypeScript exports, serialized
  render-node fields, or Rust public APIs.
- Minor releases may change an incubating public API when the replacement and migration are
  documented in the release notes. When practical, the previous spelling remains deprecated for
  at least one minor release.
- The JavaScript renderer, `@vizejs/fresco-native`, and `vize_fresco` use one repository version.
  Mixing versions is unsupported because the render protocol is validated as an exact contract.
- A major 1.0 release will promote named surfaces to stable status and publish any longer support
  window before that release ships.

Security fixes may remove unsafe behavior without a deprecation window. The release note must say
which contract changed and how to migrate safely.

## Supported extension boundaries

Extensions may depend on these documented public surfaces:

- exports from `@vizejs/fresco`, including authored SFC components, composables, theme helpers,
  stable host lifecycle, routed input, focus management, test utilities, and `FrameBuffer`;
- the closed `FrescoRenderNode` TypeScript union when building a version-matched adapter;
- public exports from `@vizejs/fresco-native` only when the extension pins the matching version;
- public modules re-exported by the `vize_fresco` crate.

Imports from `src/`, generated bundle chunks, private renderer helpers, NAPI implementation
modules, and non-public Rust modules are unsupported. Moving or changing them is not a breaking
public API change.

## Compatibility levels

| Level        | Meaning                                                                  |
| ------------ | ------------------------------------------------------------------------ |
| Stable       | Reserved for post-1.0 surfaces with a published deprecation window.      |
| Incubating   | Documented and tested, but may change in a minor release with migration. |
| Experimental | Explicit opt-in; may change or be removed in any release.                |
| Private      | Repository implementation detail with no compatibility guarantee.        |

All current Fresco package and crate APIs are incubating unless their documentation explicitly
marks them experimental or private.

## Required extension checks

An extension should test against the lowest and highest Fresco versions it claims to support. At a
minimum, verify:

1. TypeScript compilation against public package exports.
2. Native render payload rejection for unknown node kinds or mismatched fields.
3. Runtime mount, update, unmount, and terminal restoration.
4. Keyboard, focus, resize, mouse, paste, and degraded-terminal behavior used by the extension.
5. Package size and platform artifact availability for every published target.

Fresco's own CI verifies the checked-in native declarations against Rust generation and compiles an
isolated consumer before publication.
