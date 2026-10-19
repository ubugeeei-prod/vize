// Vapor parity runner (#8859). Reads `{ corpora: [{ name, cases }] }` on stdin,
// where each case carries Vize's compiled output, and prints which cases mount
// or hydrate differently from `@vue/compiler-vapor` under the same runtime.
// `crates/vize_atelier_vapor/tests/vapor_parity.rs` drives it.
import { stdoutLog } from "./quiet.ts";
import { readFileSync, writeSync } from "node:fs";
import { hydrateDiffers } from "./hydrate.ts";
import { mountDiffers, type Compiled } from "./mount.ts";
import { vueVersion } from "./runtime.ts";

interface Input {
  corpora: { name: string; cases: Compiled[] }[];
}

const input = JSON.parse(readFileSync(0, "utf8")) as Input;

const corpora = [];
for (const { name, cases } of input.corpora) {
  const mount: string[] = [];
  const hydrate: string[] = [];
  let hydrated = 0;
  for (const compiled of cases) {
    if (await mountDiffers(compiled)) mount.push(compiled.id);
    const differs = await hydrateDiffers(compiled);
    if (differs !== undefined) hydrated++;
    if (differs) hydrate.push(compiled.id);
  }
  corpora.push({
    name,
    mount: { cases: cases.length, different: mount },
    hydrate: { cases: hydrated, different: hydrate },
  });
}
console.log = stdoutLog;
writeSync(1, `${JSON.stringify({ vue: vueVersion, corpora })}\n`);
process.exit(0);
