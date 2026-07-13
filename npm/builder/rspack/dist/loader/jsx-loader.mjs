import { c as e } from "../utils-CFK8mca_.mjs";
import { r as t } from "../compiler-C710IOIZ.mjs";
function n(e) {
  let n = this.async(),
    i = this.getOptions(),
    a = this.resourcePath;
  if ((this.addDependency(a), !r(a, i))) {
    (this.emitWarning(
      Error(
        `[vize] File is filtered out by loader options include/exclude: ${a}. Passing through source unchanged.`,
      ),
    ),
      n(null, e));
    return;
  }
  try {
    let r = i.sourceMap ?? this.sourceMap ?? !0,
      {
        code: o,
        map: s,
        warnings: c,
      } = t(a, e, { jsxMode: i.jsxMode, vapor: i.vapor ?? !1, sourceMap: r });
    for (let e of c) this.emitWarning(Error(`[vize] ${e}`));
    n(null, o, s ? JSON.parse(s) : void 0);
  } catch (e) {
    n(e);
  }
}
function r(t, n) {
  return !(!e(t, n.include, !0) || e(t, n.exclude, !1));
}
export { n as default };
