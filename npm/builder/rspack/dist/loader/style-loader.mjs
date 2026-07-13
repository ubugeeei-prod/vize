import { a as e } from "../utils-CFK8mca_.mjs";
import t from "node:path";
import n from "node:fs/promises";
function r(r) {
  let i = this.async(),
    { resourceQuery: a, resourcePath: o } = this;
  if (!a) {
    i(null, r);
    return;
  }
  let s = new URLSearchParams(a.slice(1));
  if (s.get(`type`) !== `style`) {
    i(null, r);
    return;
  }
  let c = parseInt(s.get(`index`) || `0`, 10);
  this.addDependency(o);
  let l = e(r)[c];
  if (!l) {
    (this.emitError(Error(`[vize] Style block at index ${c} not found in ${o}`)), i(null, ``));
    return;
  }
  if (l.src) {
    let e = t.resolve(t.dirname(o), l.src);
    (this.addDependency(e),
      n
        .readFile(e, `utf-8`)
        .then((e) => {
          i(null, e);
        })
        .catch(() => {
          (this.emitWarning(
            Error(`[vize] <style src> target not found: ${l.src} (resolved: ${e}) in ${o}`),
          ),
            i(null, ``));
        }));
    return;
  }
  i(null, l.content);
}
export { r as default };
