import { d as e, n as t, o as n, u as r } from "./utils-CFK8mca_.mjs";
import * as i from "@vizejs/native";
import { rewriteSfcTemplateAssetReferences as a } from "@vizejs/native";
import o from "node:path";
import { createHash as s } from "node:crypto";
import { parseSync as c } from "oxc-parser";
function l(e) {
  return `
/* hot reload */
if (module.hot) {
  _sfc_main.__hmrId = "${e}"
  const api = __VUE_HMR_RUNTIME__
  module.hot.accept()
  if (!api.createRecord('${e}', _sfc_main)) {
    api.reload('${e}', _sfc_main)
  }
}`;
}
function u(e, t, n, r) {
  return `
if (module.hot) {
  module.hot.accept(${t}, () => {
    _sfc_main.__cssModules[${JSON.stringify(r)}] = ${n}
    __VUE_HMR_RUNTIME__.rerender("${e}")
  })
}`;
}
const d = `_sfc_main`;
function f(e) {
  return typeof e == `object` && !!e && typeof e.type == `string`;
}
function p(e) {
  return typeof e?.start == `number` ? e.start : null;
}
function m(e) {
  return f(e) && typeof e.name == `string` ? e.name : null;
}
function h(e) {
  try {
    let t = c(`vize-rspack-output.tsx`, e);
    if (typeof t == `object` && t) {
      let e = t.errors;
      if (Array.isArray(e) && e.length > 0) return null;
      let n = t.program;
      if (f(n)) return n;
    }
    return f(t) ? t : null;
  } catch {
    return null;
  }
}
function g(e) {
  return !e || !Array.isArray(e.body) ? [] : e.body.filter(f);
}
function _(e, t) {
  return m(e) === t;
}
function v(e) {
  return (Array.isArray(e.declarations) ? e.declarations : [])
    .filter(f)
    .map((e) => (f(e.id) ? m(e.id) : null))
    .filter((e) => e != null);
}
function y(e) {
  return g(e).find((e) => e.type === `ExportDefaultDeclaration`) ?? null;
}
function b(e, t) {
  let n = p(t);
  if (n == null) return null;
  let r = /^export\s+default\b/.exec(e.slice(n));
  return r ? n + r[0].length : null;
}
function x(e) {
  let t = h(e),
    n = g(t);
  return {
    hasDefaultExport: y(t) != null,
    hasSfcMainDefined: n.some((e) => e.type === `VariableDeclaration` && v(e).includes(d)),
  };
}
function S(e) {
  let t = y(h(e)),
    n = p(t),
    r = t ? b(e, t) : null;
  return n == null || r == null ? e : `${e.slice(0, n)}const ${d} =${e.slice(r)}`;
}
function C(e, t, n = {}) {
  let r = y(h(e)),
    i = f(r?.declaration) ? r.declaration : null,
    a = p(r),
    o = typeof r?.end == `number` ? r.end : null;
  if (!_(i, d) || a == null) return e;
  if (n.normalizeSemicolon && o != null) {
    let n = e[o] === `;` ? o + 1 : o;
    return `${e.slice(0, a)}${t}\nexport default ${d};${e.slice(n)}`;
  }
  return `${e.slice(0, a)}${t}\n${e.slice(a)}`;
}
function w(e, t) {
  let n = e.code,
    r = e.isCustomElement;
  e.templateAssetUrls.length > 0 && (n = a(n, e.templateAssetUrls));
  let i = x(n),
    s = i.hasDefaultExport,
    c = i.hasSfcMainDefined;
  if (
    (s && !c
      ? ((n = S(n)),
        e.hasScoped && e.scopeId && (n += `\n_sfc_main.__scopeId = "data-v-${e.scopeId}";`),
        (n += `
export default _sfc_main;`))
      : s &&
        c &&
        e.hasScoped &&
        e.scopeId &&
        (n = C(n, `_sfc_main.__scopeId = "data-v-${e.scopeId}";`)),
    e.styles.length > 0)
  ) {
    if (r && e.styles.some((e) => e.module))
      throw Error(`[vize] <style module> is not supported in custom elements mode.`);
    let i = e.styles.filter((e) => e.module === !0).length;
    if (i > 1)
      throw Error(
        `[vize] Found ${i} unnamed <style module> blocks. Only one unnamed <style module> is allowed per SFC. Use named modules instead: <style module="name">`,
      );
    let a = e.styles.filter((e) => e.src || /\S/.test(e.content)),
      o = [];
    if (
      ((n =
        a.map((n) => {
          let i = [
              `vue`,
              `type=style`,
              `index=${n.index}`,
              `lang=${n.lang || `css`}`,
              ...(n.scoped ? [`scoped=${e.scopeId}`] : []),
              ...(n.module ? [`module=${typeof n.module == `string` ? n.module : `true`}`] : []),
              ...(r ? [`inline`] : []),
            ],
            a = `${t.requestPath}?${i.join(`&`)}`;
          if (r) return `import _style_${n.index} from ${JSON.stringify(a)};`;
          if (n.module) {
            let e = typeof n.module == `string` ? n.module : `$style`,
              r = `_cssModule_${n.index}`;
            return (
              o.push({ request: a, varName: r, bindingName: e }),
              t.nativeCss
                ? `import * as ${r} from ${JSON.stringify(a)};`
                : `import ${r} from ${JSON.stringify(a)};`
            );
          }
          return `import ${JSON.stringify(a)};`;
        }).join(`
`) +
        `
` +
        n),
      r)
    ) {
      let e = a.map((e) => `_style_${e.index}`).join(`,`);
      n = C(n, `_sfc_main.styles = [${e}];`, { normalizeSemicolon: !0 });
    }
    if (!r && o.length > 0) {
      let r = o.map(
          (e) =>
            `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(e.bindingName)}] = ${e.varName};`,
        ).join(`
`),
        i =
          t.hmr && e.scopeId
            ? o.map((t) => u(e.scopeId, JSON.stringify(t.request), t.varName, t.bindingName)).join(`
`)
            : ``;
      n = C(n, `${r}\n${i}`, { normalizeSemicolon: !0 });
    }
  }
  if (t.filePath && !t.isProduction) {
    let e = t.rootContext
      ? o.relative(t.rootContext, t.filePath).replace(/\\/g, `/`)
      : o.basename(t.filePath);
    n = C(n, `_sfc_main.__file = ${JSON.stringify(e)};`, { normalizeSemicolon: !0 });
  }
  if (
    (t.hmr && e.scopeId && (n = C(n, l(e.scopeId), { normalizeSemicolon: !0 })),
    e.customBlocks.length > 0)
  ) {
    let r = e.customBlocks.map((e, n) => {
      let r = [`vue`, `type=${e.type}`, `index=${n}`, ...(e.src ? [`src=true`] : [])];
      for (let [t, n] of Object.entries(e.attrs))
        t !== `src` && (n === !0 ? r.push(t) : r.push(`${t}=${n}`));
      let i = `${t.requestPath}?${r.join(`&`)}`;
      return `import block${n} from ${JSON.stringify(i)};\nif (typeof block${n} === 'function') block${n}(_sfc_main);`;
    }).join(`
`);
    n = C(n, r, { normalizeSemicolon: !0 });
  }
  return (
    e.templateAssetUrls.length > 0 &&
      (n =
        e.templateAssetUrls.map(({ url: e, varName: t }) => {
          let n = e.startsWith(`~`) ? e.slice(1) : e,
            r = n.indexOf(`#`);
          return (r >= 0 && (n = n.slice(0, r)), `import ${t} from ${JSON.stringify(n)};`);
        }).join(`
`) +
        `
` +
        n),
    n
  );
}
const { compileSfc: T } = i,
  { compileJsx: E } = i;
function D(e) {
  return e.endsWith(`.jsx`) || e.endsWith(`.tsx`);
}
function O(e, t, n) {
  return `
export const __vize_css__ = ${JSON.stringify(t)};
const __vize_css_id__ = ${JSON.stringify(`vize-style-${n}`)};
(function() {
  if (typeof document !== "undefined") {
    let style = document.getElementById(__vize_css_id__);
    if (!style) {
      style = document.createElement("style");
      style.id = __vize_css_id__;
      style.textContent = __vize_css__;
      document.head.appendChild(style);
    } else {
      style.textContent = __vize_css__;
    }
  }
})();
${e}`;
}
function k(e, t, n = {}) {
  let r = E(t, {
    filename: e,
    lang: e.endsWith(`.tsx`) ? `tsx` : `jsx`,
    jsxMode: n.jsxMode,
    vapor: n.vapor ?? !1,
    sourceMap: n.sourceMap ?? !1,
  });
  if (r.errors.length > 0)
    throw Error(
      `[vize] Compilation failed for ${e}:\n${r.errors.join(`
`)}`,
    );
  let i = (r.scopedStyles ?? []).map((e) => e.css).join(`
`),
    a = r.code,
    o = r.map ?? null;
  if (i) {
    let e = r.scopedStyles[0].scopeId.replace(/^data-v-/, ``);
    ((a = O(a, i, e)), (o = null));
  }
  return { code: a, map: o, warnings: r.warnings };
}
const A = new Map();
function j(e) {
  return s(`sha256`).update(e).digest(`hex`).slice(0, 16);
}
function M() {
  A.clear();
}
function N(i, a, o = {}) {
  let s = o.compilerOptions?.isTs ?? /<script[^>]*\blang=["']ts["']/.test(a),
    c = o.ssr ?? o.compilerOptions?.ssr ?? !1,
    l = o.vapor ?? o.compilerOptions?.vapor ?? !1,
    u = o.sourceMap ?? o.compilerOptions?.sourceMap ?? !0,
    d = o.isCustomElement ?? !1,
    f = o.rootContext ?? ``,
    p = o.isProduction ?? !1,
    m = o.transformAssetUrls ?? !0,
    h = `${i}:ssr=${c}:vapor=${l}:ts=${s}:map=${u}:ce=${d}:syntax=${o.compilerOptions?.templateSyntax ?? `standard`}:xic=${o.compilerOptions?.experimentalInTagComments ?? !1}:xpt=${o.compilerOptions?.experimentalPatternedTemplate ?? !1}:xss=${o.compilerOptions?.experimentalServerScript ?? !1}:root=${f}:prod=${p}:${m === !1 ? `tau=false` : m === !0 ? `tau=true` : `tau=${JSON.stringify(m)}`}`,
    g = j(a),
    _ = A.get(h);
  if (_ && _.contentHash === g) return _.result;
  let v = n(i, o.rootContext, o.isProduction, a),
    y = T(a, {
      ...o.compilerOptions,
      filename: i,
      sourceMap: o.sourceMap ?? o.compilerOptions?.sourceMap ?? !0,
      ssr: c,
      vapor: l,
      isTs: s,
      scopeId: `data-v-${v}`,
    }),
    b = t(a, m),
    x = {
      code: y.code,
      css: y.css,
      errors: y.errors,
      warnings: y.warnings,
      scopeId: v,
      hasScoped: y.hasScoped,
      styles: y.styles.map(e),
      customBlocks: y.customBlocks.map(r),
      isCustomElement: d,
      templateAssetUrls: b,
      macroArtifacts: y.macroArtifacts ?? [],
    };
  return (x.errors.length === 0 && A.set(h, { contentHash: g, result: x }), x);
}
export { w as a, D as i, N as n, l as o, k as r, M as t };
