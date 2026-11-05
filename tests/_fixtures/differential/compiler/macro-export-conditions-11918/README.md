Reported by Jimmy Chen ([@Q16solver](https://github.com/Q16solver)) in
[#11918](https://github.com/ubugeeei-prod/vize/issues/11918), from the
[Esposter db-schema package](https://github.com/Esposter/Esposter/blob/48cc3c0946de6ac8cd582d9a0f2a163aca77b333/packages/db-schema/package.json#L44).

The package has a `source` export and an unbuilt declaration path. Its importing
component belongs to a bundler tsconfig with `customConditions: ["source"]`.
The macro compiler must resolve the same source interface as TypeScript and emit
`label: String` (required) and `count: Number` (optional).

The pinned complete module was observed from public Vize 0.691.0 using a paths
alias to the exact same source file. The unmodified condition-based project
fails on that version. The regression compiles the unmodified project, compares
the complete module, and separately checks inherited conditions, declaration
order, nested/array targets, wildcard/exact/null subpaths, condition changes,
and inactive conditions with exact runtime-prop/error oracles.
The route matrix was independently checked with TypeScript 6.0.3 and the Vue
3.5.35 compiler: an explicit null stops array fallback, a missing file permits
the next array entry, and inactive exports cannot expose top-level types.

For release acceptance, materialize the five authenticated project files using
`tests/differential/compiler-project.ts`, then run a fresh installed `vize`
package's `vize build Layout.vue --ssr --no-config -o out`. Compare the complete
`out/Layout.js` with `module.expected.txt`. Repeat without `customConditions`:
the build must fail with `Cannot resolve imported macro type Props from lib`
(diagnostic identifiers are backtick quoted). The original report additionally
qualifies `@vizejs/vite-plugin` with Vite and `@vizejs/unplugin` with Rolldown;
both builds must complete, and the official Vue compiler is the reference.
