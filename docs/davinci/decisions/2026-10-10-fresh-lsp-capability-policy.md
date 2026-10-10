# Fresh project LSP capability policy (2026-10-10)

Issue: [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

The authorized successor changes only projects without dedicated `vize.config.*`:
document, range and on-type formatting are advertised, and the existing
`experimental.vize.jsxTypecheck` field becomes `true`. The on-type triggers remain
`;`, `}` and newline. Every other capability and the entire auto-insertion
extension retain the original contract. Initialization flags still override
project formatting settings.

`tests/_fixtures/differential/lsp/config-defaults-8371/` preserves the complete
original 350-line capability test and original JSX component test at
`93f10aacc174999098760cf5eea4d5165f9384f1`, their hashes, and the complete historical
capability value. The fresh value is independently authored from that historical
value by exactly those four field changes. A fixed custody-manifest hash and a
whole-object equality law prevent an unrelated rebaseline. Existing feature
controls remain unchanged. The original no-config Zed archive is unchanged;
its separate formatting successor does not authorize a broader golden refresh.

A dedicated empty configuration retains the complete historical packet:
formatting providers are absent and JSX type checking is off. Explicit false
values retain the same behavior in an ordinary Vite project:

```ts
// vite.config.ts
export default {
  vize: {
    languageServer: { formatting: false },
    typeChecker: { jsxTypecheck: false },
  },
};
```

Vue JSX ownership is a project setting, not a guess from JSX syntax. React-owned
projects must explicitly disable Vize JSX type checking. A single project that
mixes React and Vue JSX cannot apply one framework interpretation safely to both;
disable Vize JSX checking there and retain each framework's checker. Vue SFC
checking remains available. Separate package settings must be checked through the
actual nested-workspace source path before claiming independent package routing.

Fresh TSX and checkJs JSX witnesses reuse the original Counter, invalid consumer,
repaired consumer and intrinsic-element bytes with real Vue declarations. The
original prop diagnostic law locates `count` at line 1, characters 29–34. Tests
compare entire diagnostics publications, including version, and normalize only
the temporary workspace URI. Notification waiters do not select by diagnostic
contents. No dedicated configuration, synthetic Vue shim or explicit JSX opt-in
is added to these witnesses.

The new fresh-project witness explicitly binds the existing receipted current
source executable and retains all raw protocol streams. Selected PR tooling does
not globally require source binding, so the generic session may otherwise select
a cached debug executable before the current CI executable. The original two
invalid publications returned empty diagnostics in source Actions at `22b145`;
this is retained as a failed observation. A fresh bound run must distinguish the
launch issue from a native diagnostic regression without changing any input,
expected packet, or notification predicate.

Source inspection also found an actual bottom-layer default bug: the LSP facade
used the strict CLI selection loader for an existing editor directory without
configuration, marked the snapshot invalid, and returned before applying fresh
defaults. Promote the already reviewed private editor entry point from the
nested-context layer into the first configuration layer. Only existing editor
directories permit absent configuration; malformed input, missing paths, and
explicit CLI selection retain their original refusal. An additive whole snapshot
and strict-CLI control covers that distinction at the owning layer. The bound
source run must now verify the original complete diagnostic packets.

The local observations distinguish capability/prop witnesses from release
qualification. The original complete default-capability test still reproduces
the intentional policy difference; the fresh successor and historical/explicit
controls pass with the supplied native LSP binary. These are local observations,
not an exact-head Actions source receipt. Fresh default admission remains held
until [typed slot parameter retention](https://github.com/ubugeeei-prod/vize/pull/8428),
[nested JSX lowering](https://github.com/ubugeeei-prod/vize/pull/8457), and the
original Ant Design Vue semantic-loss source repair are qualified without
changing their authored inputs or expectations. Protected queue validation,
actual merge and release remain required.

## React-owned projects

For a React-owned project, explicitly disable Vize's Vue JSX checker in the existing Vite config:

```ts
// vite.config.ts
export default {
  vize: { typeChecker: { jsxTypecheck: false } },
};
```

Vize does not infer framework ownership from JSX syntax. If one project mixes React and Vue JSX,
disable Vize JSX checking and use each framework's checker for those files. Vue SFC checking remains
available. See the [fresh default qualification record](https://github.com/ubugeeei-prod/vize/blob/main/docs/davinci/decisions/2026-10-10-fresh-lsp-capability-policy.md)
for the unchanged historical controls and remaining native gates.
