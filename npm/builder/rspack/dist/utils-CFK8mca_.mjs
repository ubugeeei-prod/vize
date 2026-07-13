import {
  collectSfcTemplateAssetUrls as e,
  extractSfcCustomBlocks as t,
  extractSfcSrcInfo as n,
  extractSfcStyleBlocks as r,
  generateSfcScopeId as i,
  scopeViteCssForPipeline as a,
  stripSfcScopedCssComments as o,
} from "@vizejs/native";
function s(e, t, n, r) {
  return i(e, t, n, r);
}
function c(e) {
  return r(e).map(v);
}
function l(e) {
  return o(e);
}
function u(e, t) {
  return a(e, t.startsWith(`data-v-`) ? t : `data-v-${t}`);
}
function d(e) {
  return t(e).map(y);
}
function f(e) {
  let t = n(e);
  return { scriptSrc: t.scriptSrc ?? null, templateSrc: t.templateSrc ?? null };
}
function p(e, t, n) {
  let r = e;
  return (
    t !== null &&
      (r = r.replace(
        /(<script)([^>]*)\bsrc=["'][^"']+["']([^>]*>)[\s\S]*?(<\/script>)/i,
        (e, n, r, i, a) => `${n}${(r + i).replace(/\bsrc=["'][^"']+["']\s*/g, ``)}\n${t}\n${a}`,
      )),
    n !== null &&
      (r = r.replace(
        /(<template)([^>]*)\bsrc=["'][^"']+["']([^>]*>)[\s\S]*?(<\/template>)/i,
        (e, t, r, i, a) => `${t}${(r + i).replace(/\bsrc=["'][^"']+["']\s*/g, ``)}\n${n}\n${a}`,
      )),
    r
  );
}
function m(e, t, n) {
  if (!t) return n;
  let r = e.replace(/\\/g, `/`);
  return (Array.isArray(t) ? t : [t]).some((t) =>
    typeof t == `string` ? r.includes(t) || e.includes(t) : h(t, r),
  );
}
function h(e, t) {
  e.lastIndex = 0;
  let n = e.test(t);
  return ((e.lastIndex = 0), n);
}
Object.freeze({
  img: [`src`],
  video: [`src`, `poster`],
  source: [`src`],
  image: [`xlink:href`, `href`],
  use: [`xlink:href`, `href`],
});
function g(t, n) {
  return n === !1 ? [] : e(t, _(n));
}
function _(e) {
  if (!(e == null || e === !0)) return Object.entries(e).map(([e, t]) => ({ tag: e, attrs: t }));
}
function v(e) {
  return {
    content: e.content,
    src: e.src ?? null,
    lang: e.lang ?? null,
    scoped: e.scoped,
    module: e.module ? (e.moduleName ?? !0) : !1,
    index: e.index,
  };
}
function y(e) {
  let t = {};
  for (let n of e.attrs) t[n.name] = n.value ?? !0;
  return { type: e.blockType, content: e.content, src: e.src ?? null, attrs: t, index: e.index };
}
export { c as a, m as c, v as d, f as i, l, g as n, s as o, d as r, p as s, u as t, y as u };
