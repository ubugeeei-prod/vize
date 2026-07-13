import { l as e } from "../utils-CFK8mca_.mjs";
import * as t from "@vizejs/native";
const { compileCss: n } = t;
function r(t) {
  let r = this.async(),
    { resourceQuery: i, resourcePath: a } = this;
  if (!i) {
    r(null, t);
    return;
  }
  let o = new URLSearchParams(i.slice(1)).get(`scoped`);
  if (!o) {
    r(null, t);
    return;
  }
  let s = `data-v-${o}`,
    c = n(e(t), { filename: a, scoped: !0, scopeId: s });
  for (let e of c.errors) this.emitError(Error(`[vize] ${e}`));
  for (let e of c.warnings) this.emitWarning(Error(`[vize] ${e}`));
  r(null, c.code);
}
export { r as default };
