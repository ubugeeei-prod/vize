# Editor JavaScript config discovery

The user's no-Vize-config alias report requires JavaScript coverage as well as
the original TypeScript [#7991](https://github.com/ubugeeei-prod/vize/issues/7991)
case. Before the package-import child can qualify ordinary JS and setup-JS,
its editor mirror must use the authored `jsconfig.json` compiler options.

Native startup and the original native project resolver already search the
nearest directory with either `tsconfig.json` or `jsconfig.json`, preferring
TypeScript when both exist in that same directory. The editor alias-context
builder searched only for `tsconfig.json`, losing JavaScript `checkJs`, JSDoc
and module-resolution authority before mirroring. Select the actual nearest
config file and derive its directory without changing explicit configured
path/root precedence or effective project-reference ownership. Add no alias,
backend query, process, input scan or pipeline stage.

The bottom provider slice retains a new complete relative-import JS project:
ordinary `.js`, setup-JS SFC, JSDoc string parameter/return and `checkJs`.
Pure laws cover a nested `jsconfig.json` beneath a parent TypeScript config,
same-directory TypeScript precedence and explicit config precedence. Actual
source-built stdio and pinned native 7.0.2 controls compare complete clean,
wrong-argument and missing-module vectors under each config name, plus a genuine
JavaScript-config variant omitting `allowJs` to require native implicit-default
authority rather than infer it from an explicit flag. A conflicting
same-directory JavaScript config disables `checkJs`; the selected TypeScript
config must still diagnose the ordinary JS wrong argument. Native bare-script
suggestions are independently retained and every config publication is checked
as a whole envelope; publication multiplicity is asynchronous, all frames are
captured, and no diagnostic is filtered. Native shutdown acknowledgement and
stdin EOF, Vize standard exit and both child-close streams require success.

The existing CLI runner also names only `tsconfig.json` in default discovery;
this bounded editor provider does not claim that separate CLI discovery path
supports a JavaScript-only config. The private-import child independently
qualifies CLI JS using an authored allowJs/checkJs TypeScript config, then
editor/native JS with both genuine config authorities. Nuxt fixture coverage
is owned separately and no generated Nuxt mapping is guessed here.

Use a real native Stack: this config provider is the parent of the private
package-import child. Fresh exact-head source/native Actions, protected full
suites/instruction ceilings, actual signed prefix merge and publication remain
pending; no independent layer auto-merge or old runtime credit applies.
