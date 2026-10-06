# Configless CLI discovery of JavaScript projects

These are authored regression controls. No external reporter source or measured
runtime is claimed. `message.js.txt` is ordinary JavaScript with a JSDoc type;
there is no suffix conversion, TypeScript rewrite, provider stub, or Vize config.
The original implicit `jsconfig.json` omits `allowJs`, `skipLibCheck`, `noEmit`,
and `maxNodeModuleJsDepth`. The explicit-false inverse retains `checkJs: true`.
Only the repaired source changes the assigned value. The separate authored
TypeScript file is a control for mixed workspace program grouping.

`corpus.json` pins every project carrier and the complete authenticated native
source files from TypeScript Go commit `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`.
The ordinary installed SDK must actually be version 7.0.2; those source files
are reference evidence and are never executed as a substituted provider.
`tsconfigparsing.go` lines 887–897 define filename-dependent JavaScript defaults,
909–915 retain authored overrides, and 1039–1160 order own defaults and extends.
`compileroptions.go` lines 282–286 give explicit `allowJs` precedence over
`checkJs`. Upstream source: https://github.com/microsoft/typescript-go/tree/2bd066d87f5bafd315be9f40889d0a60b9e58e0b

The always-selected tooling law must use the current receipted source CLI and
physical installed native executable, preserve all argv, exit state, raw
stdout/stderr, full parsed CLI reports including programs/options, and run
ordinary default discovery without `--no-config` or an implicit `--tsconfig`.
Explicit jsconfig selection, same-directory tsconfig precedence, nearest
package selection, explicit-false denial, repair, and mixed jsconfig/tsconfig
program ownership remain separate controls. No runtime credit, performance
claim, or release credit follows from this source preparation.
