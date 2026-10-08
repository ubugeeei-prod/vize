# Shared-ancestor tsconfig inheritance (#3984)

The complete input manifest keeps both sibling configs, their common base,
selected and excluded TS/JS roots, an extra authored Vue root, broken and
repaired source, and an independent declaration-only consumer. Later extends
entries inherit the common base before overriding earlier entries. The original
CLI loader instead suppressed the second visit, selected the first subtree, and
used its declaration directory and disabled maps.

The external oracle is official TypeScript 6.0.3. Run `tsc -p config.json
--showConfig` on the manifest's plain TS/JS project (rename `tsconfig.json` to
`config.json` for the preserved command); the full config result is retained.
The declaration and map oracles come from the repaired plain TS source with
`--noEmit false --emitDeclarationOnly true --pretty false`. The Vue root has its
own complete CLI and declaration-only consumer laws; the stock config oracle
does not establish native Vue mapper acceptance.

The native-required public CLI laws compare all authored file diagnostics, the
whole program root/config/options/input set, declaration output paths, the full
plain TS declaration and map, Vue map source identity, and clean/broken/repaired
consumers after only emitted files are copied into a separate package. Original
source is removed before that consumer runs. Controls retain reverse sibling
order, local false/empty overrides, output-directory precedence, and cycle/error
behavior. No old corpus or instruction limit changes.
