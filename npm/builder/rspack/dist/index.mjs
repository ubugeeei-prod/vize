import { a as e, c as t, i as n, o as r, r as i, s as a, t as o } from "./utils-CFK8mca_.mjs";
import { n as s, r as c, t as l } from "./loader-CHpYOm39.mjs";
import { a as u, i as d, n as f, o as p, r as m, t as h } from "./compiler-C710IOIZ.mjs";
import g from "./loader/jsx-loader.mjs";
import _ from "./loader/style-loader.mjs";
import v from "./loader/scope-loader.mjs";
const y = `@vizejs/rspack-plugin/style-loader`,
  b = `@vizejs/rspack-plugin/scope-loader`,
  x = {
    "\\.css$": `css`,
    "\\.scss$": `scss`,
    "\\.sass$": `sass`,
    "\\.less$": `less`,
    "\\.styl(us)?$": `styl`,
  };
function S(e, t) {
  let n = [],
    r = e.findIndex((e) => e !== `...` && C(e));
  if (r === -1) return { applied: !1, clonedCount: 0, warnings: n };
  let i = e[r];
  if (i.oneOf) return { applied: !1, clonedCount: 0, warnings: n };
  let a = [];
  for (let t = 0; t < e.length; t++) {
    if (t === r) continue;
    let n = e[t];
    if (n === `...`) continue;
    let i = E(n);
    i && a.push({ index: t, rule: n, lang: i });
  }
  let o = [];
  for (let e of a) o.push(...D(e.rule, e.lang, t));
  o.some(
    (e) =>
      e.resourceQuery instanceof RegExp && e.resourceQuery.test(`vue&type=style&index=0&lang=css`),
  ) || o.push(...O(t));
  let s = { use: k(M(i), t) },
    c = [...o, s];
  e[r] = { test: i.test, oneOf: c };
  for (let e of a) A(e.rule);
  return { applied: !0, clonedCount: o.length, warnings: n };
}
function C(e) {
  return T(e.test)
    ? M(e).some((e) => {
        let t = typeof e == `string` ? e : e.loader;
        return t ? w(t) : !1;
      })
    : !1;
}
function w(e) {
  let t = e.replaceAll(`\\`, `/`);
  return (
    e === `@vizejs/rspack-plugin/loader` ||
    ((t.includes(`@vizejs/rspack-plugin`) || t.includes(`rspack-vize-plugin`)) &&
      /\/dist\/loader\/index\.[cm]?js$/.test(t))
  );
}
function T(e) {
  return e
    ? e instanceof RegExp
      ? e.test(`App.vue`) || e.test(`foo.vue`)
      : typeof e == `string`
        ? e.includes(`.vue`)
        : !1
    : !1;
}
function E(e) {
  let t = e.test;
  if (!t || !(t instanceof RegExp)) return null;
  let n = t.source;
  for (let [e, t] of Object.entries(x)) if (n.includes(e) || n === e) return t;
  return t.test(`foo.css`) && !t.test(`foo.vue`)
    ? `css`
    : t.test(`foo.scss`) && !t.test(`foo.vue`)
      ? `scss`
      : t.test(`foo.sass`) && !t.test(`foo.vue`)
        ? `sass`
        : t.test(`foo.less`) && !t.test(`foo.vue`)
          ? `less`
          : t.test(`foo.styl`) && !t.test(`foo.vue`)
            ? `styl`
            : null;
}
function D(e, t, n) {
  let r = M(e);
  if (r.length === 0) return [];
  let i = [{ loader: b }, ...N(r), { loader: y }],
    a = (t, n) => {
      let r = { resourceQuery: t, use: N(i) };
      return (e.type ? (r.type = e.type) : n && (r.type = n), r);
    };
  return n
    ? [
        a(RegExp(`(?=.*type=style)(?=.*lang=${t})(?=.*module=)`), `css/module`),
        a(RegExp(`(?=.*type=style)(?=.*lang=${t})`), `css/auto`),
      ]
    : [a(RegExp(`(?=.*type=style)(?=.*lang=${t})`))];
}
function O(e) {
  return e
    ? [
        {
          resourceQuery: /(?=.*type=style)(?=.*lang=css)(?=.*module=)/,
          type: `css/module`,
          use: [{ loader: b }, { loader: y }],
        },
        {
          resourceQuery: /(?=.*type=style)(?=.*lang=css)/,
          type: `css/auto`,
          use: [{ loader: b }, { loader: y }],
        },
      ]
    : [
        {
          resourceQuery: /(?=.*type=style)(?=.*lang=css)/,
          type: `javascript/auto`,
          use: [{ loader: b }, { loader: y }],
        },
      ];
}
function k(e, t) {
  return N(e).map((e) => {
    if (typeof e == `string`) return w(e) ? { loader: e, options: { css: { native: t } } } : e;
    if (typeof e != `object` || !e) return e;
    let n = e.loader;
    if (!n || !w(n)) return e;
    let r = e.options;
    if (!r || typeof r != `object` || Array.isArray(r))
      return { ...e, options: { css: { native: t } } };
    let i = r,
      a = i.css;
    return !a || typeof a != `object` || Array.isArray(a)
      ? { ...e, options: { ...i, css: { native: t } } }
      : { ...e, options: { ...i, css: { ...a, native: t } } };
  });
}
function A(e) {
  let t = e.resourceQuery;
  if (t)
    return (
      typeof t == `object` && !Array.isArray(t) && !(t instanceof RegExp) && `not` in t, void 0
    );
  e.resourceQuery = { not: [/vue/] };
}
function j(e) {
  return e ? (Array.isArray(e) ? e : [e]) : [];
}
function M(e) {
  let t = j(e.use);
  if (t.length > 0) return t;
  let n = e.loader;
  if (n) {
    let t = e.options;
    return t ? [{ loader: n, options: t }] : [n];
  }
  return [];
}
function N(e) {
  return e.map((e) => {
    if (typeof e == `string`) return e;
    if (typeof e == `object` && e) {
      let t = { ...e };
      return (
        `options` in e &&
          e.options &&
          typeof e.options == `object` &&
          (t.options = { ...e.options }),
        t
      );
    }
    return e;
  });
}
var P = class e {
  static name = `VizePlugin`;
  options;
  constructor(e = {}) {
    this.options = e;
  }
  apply(t) {
    let n = t.getInfrastructureLogger(e.name),
      r = this.options.isProduction ?? t.options.mode === `production`;
    this.options.vapor && !r && n.debug(`Vapor mode is enabled.`);
    let i = s(t.options),
      a = t.webpack?.rspackVersion,
      o = c(this.options.css?.native, t.options, a);
    if (
      (this.options.css?.native &&
        i === `disabled` &&
        n.warn("`css.native: true` is set but `experiments.css` is not enabled in rspack config."),
      this.options.autoRules ?? !0)
    ) {
      let e = t.options.module?.rules;
      if (e) {
        let t = S(e, o);
        t.applied &&
          n.debug(`Auto-injected ${t.clonedCount} style rule(s) for Vue SFC sub-requests.`);
        for (let e of t.warnings) n.warn(e);
      }
    }
    if (this.options.typescript ?? !0) {
      let e = t.options.module?.rules;
      e &&
        (e.some((e) => {
          if (e === `...` || typeof e != `object` || !e) return !1;
          let t = e;
          if (t.enforce !== `post`) return !1;
          let n = t.test;
          return n instanceof RegExp
            ? n.test(`App.vue`)
            : typeof n == `string`
              ? n.includes(`.vue`)
              : !1;
        }) ||
          (e.push({
            test: /\.vue$/,
            resourceQuery: { not: [/type=/] },
            enforce: `post`,
            loader: `builtin:swc-loader`,
            options: { jsc: { parser: { syntax: `typescript` } } },
            type: `javascript/auto`,
          }),
          n.debug(`Auto-injected TypeScript post-processing rule for .vue files.`)));
    }
    let { DefinePlugin: l } = t.webpack,
      u = new Set();
    for (let e of t.options.plugins ?? []) {
      let t = e?.definitions;
      if (t) for (let e of Object.keys(t)) u.add(e);
    }
    let d = {};
    (u.has(`__VUE_OPTIONS_API__`) || (d.__VUE_OPTIONS_API__ = JSON.stringify(!0)),
      u.has(`__VUE_PROD_DEVTOOLS__`) || (d.__VUE_PROD_DEVTOOLS__ = JSON.stringify(!r)),
      u.has(`__VUE_PROD_HYDRATION_MISMATCH_DETAILS__`) ||
        (d.__VUE_PROD_HYDRATION_MISMATCH_DETAILS__ = JSON.stringify(!r)),
      Object.keys(d).length > 0 && new l(d).apply(t),
      r ||
        t.hooks.watchRun.tap(e.name, (e) => {
          let t = e.modifiedFiles,
            r = e.removedFiles;
          if (t)
            for (let e of t)
              e.endsWith(`.vue`) && this.shouldHandleFile(e) && n.debug(`Vue file changed: ${e}`);
          if (r)
            for (let e of r)
              e.endsWith(`.vue`) && this.shouldHandleFile(e) && n.debug(`Vue file removed: ${e}`);
        }));
  }
  shouldHandleFile(e) {
    return !(!t(e, this.options.include, !0) || t(e, this.options.exclude, !1));
  }
};
export {
  P as VizePlugin,
  o as addScopeToCssFallback,
  S as applyRuleCloning,
  h as clearCompilationCache,
  f as compileFile,
  m as compileJsxModule,
  i as extractCustomBlocks,
  n as extractSrcInfo,
  e as extractStyleBlocks,
  p as genHotReloadCode,
  u as generateOutput,
  r as generateScopeId,
  a as inlineSrcBlocks,
  d as isJsxFile,
  t as matchesPattern,
  g as vizeJsxLoader,
  l as vizeLoader,
  v as vizeScopeLoader,
  _ as vizeStyleLoader,
};
