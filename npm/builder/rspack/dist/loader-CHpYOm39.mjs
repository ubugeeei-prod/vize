import { c as e, i as t, r as n, s as r } from "./utils-CFK8mca_.mjs";
import { a as i, n as a } from "./compiler-C710IOIZ.mjs";
import o from "node:fs";
import s from "node:path";
function c(e) {
  let t = e?.experiments;
  return !t || typeof t != `object` || !Object.prototype.hasOwnProperty.call(t, `css`)
    ? `unavailable`
    : t.css
      ? `enabled`
      : `disabled`;
}
function l(e) {
  if (typeof e != `string`) return null;
  let t = Number.parseInt(e, 10);
  return Number.isNaN(t) ? null : t;
}
function u(e, t, n) {
  if (e != null) return e;
  let r = c(t);
  if (r !== `unavailable`) return r === `enabled`;
  let i = l(n);
  return i != null && i >= 2;
}
const d = /\.ce\.vue$/;
function f(e) {
  let c = this.async(),
    l = this.getOptions(),
    u = this.resourcePath,
    d = this.resourceQuery,
    f = g(this, u),
    _ = this.mode === `production` || process.env.NODE_ENV === `production`,
    v = !(l.ssr ?? !1) && !_ && l.hotReload !== !1,
    y = m(this, l);
  if ((this.addDependency(u), d?.includes(`type=style`))) {
    c(
      Error(
        `[vize] Main loader received style sub-request: ${u}${d}. Use module.rules[].oneOf with resourceQuery branches so style requests are handled by @vizejs/rspack-plugin/style-loader.`,
      ),
    );
    return;
  }
  if (d && d.includes(`vue`) && d.includes(`type=`) && !d.includes(`type=style`)) {
    let t = new URLSearchParams(d.slice(1)),
      r = t.get(`type`);
    if (r && r !== `style`) {
      let i = parseInt(t.get(`index`) || `0`, 10),
        a = n(e)[i];
      if (a) {
        if (a.src) {
          let e = s.resolve(s.dirname(u), a.src);
          this.addDependency(e);
          try {
            c(null, o.readFileSync(e, `utf-8`));
          } catch {
            c(
              Error(`[vize] Custom block <${r} src="${a.src}"> not found (resolved: ${e}) in ${u}`),
            );
          }
          return;
        }
        c(null, a.content);
      } else c(null, ``);
      return;
    }
  }
  if (!p(u, l)) {
    (this.emitWarning(
      Error(
        `[vize] File is filtered out by loader options include/exclude: ${u}. Passing through source unchanged.`,
      ),
    ),
      c(null, e));
    return;
  }
  try {
    let n = h(u, l.customElement),
      d = t(e),
      p = e;
    if (d.scriptSrc) {
      let e = s.resolve(s.dirname(u), d.scriptSrc);
      this.addDependency(e);
      try {
        let t = o.readFileSync(e, `utf-8`);
        p = r(p, t, null);
      } catch {
        c(Error(`[vize] <script src="${d.scriptSrc}"> not found (resolved: ${e}) in ${u}`));
        return;
      }
    }
    if (d.templateSrc) {
      let e = s.resolve(s.dirname(u), d.templateSrc);
      this.addDependency(e);
      try {
        let t = o.readFileSync(e, `utf-8`);
        p = r(p, null, t);
      } catch {
        c(Error(`[vize] <template src="${d.templateSrc}"> not found (resolved: ${e}) in ${u}`));
        return;
      }
    }
    let m = a(u, p, {
      sourceMap: l.sourceMap ?? this.sourceMap ?? !0,
      ssr: l.ssr ?? !1,
      vapor: l.vapor ?? !1,
      compilerOptions: l.compilerOptions,
      isCustomElement: n,
      rootContext: this.rootContext,
      isProduction: _,
      transformAssetUrls: l.transformAssetUrls,
    });
    for (let e of m.warnings) this.emitWarning(Error(`[vize] ${e}`));
    if (m.errors.length > 0) {
      for (let e of m.errors) this.emitError(Error(`[vize] ${e}`));
      let e = m.errors.join(`\\n`);
      c(Error(`[vize] Compilation failed for ${u}:\n${e}`));
      return;
    }
    c(
      null,
      i(m, {
        requestPath: f,
        hmr: v,
        filePath: u,
        isProduction: _,
        rootContext: this.rootContext,
        nativeCss: y,
      }),
    );
  } catch (e) {
    c(e);
  }
}
function p(t, n) {
  return !(!e(t, n.include, !0) || e(t, n.exclude, !1));
}
function m(e, t) {
  let n = e._compiler;
  return u(t.css?.native, n?.options, n?.webpack?.rspackVersion);
}
function h(e, t) {
  return t === !0 ? !0 : t === !1 || t === void 0 ? d.test(e) : t.test(e);
}
function g(e, t) {
  return `./${s.basename(t)}`;
}
export { c as n, u as r, f as t };
