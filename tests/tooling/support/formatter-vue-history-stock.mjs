import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { compileFunction } from "node:vm";
import { createVueHistoryProviders } from "./formatter-vue-history-providers.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
// This is the author's complete JSON transport, not input/code normalization.
const packetSnapshot = (packet) => JSON.parse(JSON.stringify(packet));
const failure = (error) => ({ message: error.message, stack: error.stack });
export const vueHistoryArtifactSha256 =
  "bf8c553b9fd93908061acbeb8a239a0ee945c7f59209ce25d406d53ef28a3d36";
const parseOptions = { pad: false, deindent: false };
const compileOptions = { comments: true, outputSourceRange: true, whitespace: "preserve" };
export async function createVueHistoryStockObserver(repositoryRoot) {
  const { identities, c26, c27, Vue, server, stock3 } =
    await createVueHistoryProviders(repositoryRoot);

  function rawTemplate(source) {
    const match = source.match(/<template>([\s\S]*?)<\/template>/);
    if (!match) throw new Error("complete source has no plain template carrier");
    return match[1];
  }
  function legacyTemplatePacket(template, version, filename) {
    return version === "2"
      ? c26.compile(template, compileOptions)
      : c27.compileTemplate({
          source: template,
          filename,
          isProduction: true,
          prettify: false,
          compilerOptions: compileOptions,
        });
  }
  function compilePacket(source, version, id = "case") {
    if (version === "3") return stock3.compilePacket(source, id);
    if (!["2", "2.7"].includes(version)) throw new Error(`unsupported sealed version ${version}`);
    const compiler = version === "2" ? c26 : c27;
    const descriptor = compiler.parseComponent(source, parseOptions);
    if (!descriptor.template) throw new Error(`${id}: stock descriptor has no template`);
    const template = descriptor.template.content;
    return packetSnapshot({
      parseOptions,
      compileOptions,
      descriptorErrors: descriptor.errors ?? [],
      template,
      templateSha256: sha256(template),
      ...legacyTemplatePacket(template, version, "App.vue"),
    });
  }
  function captureFragments(source, version, fragments, persist) {
    const template = rawTemplate(source);
    for (const [index, match] of [...template.matchAll(/<p\b[\s\S]*?<\/p>/g)].entries()) {
      const fragment = match[0];
      const row = {
        index,
        sourceUtf16Start: match.index,
        sourceUtf16End: match.index + fragment.length,
        fragment,
        fragmentSha256: sha256(fragment),
        packet: null,
      };
      fragments.push(row);
      persist();
      row.packet = packetSnapshot(legacyTemplatePacket(fragment, version, "App.vue"));
      persist();
    }
  }
  function evaluateLegacy(packet, version, state, result) {
    const calls = (result.filterCalls = []);
    const ctx = {
      ...state,
      message: state.messageText,
      _s: (value) =>
        value == null
          ? ""
          : typeof value === "object"
            ? JSON.stringify(value, null, 2)
            : String(value),
      _v: (text) => ({ text }),
      _c: (tag, data, children) => {
        if (Array.isArray(data)) {
          children = data;
          data = null;
        }
        return { tag, attrs: data?.attrs ?? {}, children: children ?? [] };
      },
      _f:
        (name) =>
        (...args) => {
          calls.push({ name, args });
          switch (name) {
            case "format-date":
              return `${args[1]}:${String(args[0])}:${args[2]?.suffix ?? ""}`;
            case "suffix":
            case "append":
              return String(args[0]) + String(args[1]);
            case "choose":
              return `${String(args[0])}:${args[1]}:${args[2]}`;
            default:
              throw new Error(`undeclared filter ${name}`);
          }
        },
    };
    ctx._self = ctx;
    const render =
      version === "2"
        ? compileFunction(packet.render, [])
        : compileFunction(`${packet.code}\nreturn render;`, [])();
    result.vnode = render.call(ctx);
  }
  async function evaluateVue3(packet, state, result) {
    Object.assign(result, {
      domHtml: null,
      textContent: null,
      ssrHtml: null,
      domCalls: [],
      ssrCalls: [],
      warnings: [],
    });
    const render = compileFunction(packet.dom.code, ["Vue"])(Vue);
    render._rc = true;
    const ssrRender = compileFunction(packet.ssr.code, ["require"])((name) => {
      if (!["vue", "vue/server-renderer"].includes(name))
        throw new Error(`undeclared SSR import ${name}`);
      return name === "vue" ? Vue : server;
    });
    const make = (calls) => ({
      data: () => ({ ...state, message: state.messageNum }),
      methods: {
        date(value) {
          calls.push({ name: "date", args: [value] });
          return state.dateValue;
        },
        suffix(value) {
          calls.push({ name: "suffix", args: [value] });
          return state.suffixValue;
        },
      },
    });
    const host = document.createElement("div");
    document.body.append(host);
    const app = Vue.createApp({ ...make(result.domCalls), render });
    app.config.warnHandler = (message) => result.warnings.push(message);
    let mounted = false;
    try {
      app.mount(host);
      mounted = true;
      await Vue.nextTick();
      result.domHtml = host.innerHTML;
      result.textContent = host.textContent;
    } finally {
      if (mounted) app.unmount();
      host.remove();
    }
    const ssrApp = Vue.createSSRApp({ ...make(result.ssrCalls), ssrRender });
    ssrApp.config.warnHandler = (message) => result.warnings.push(message);
    result.ssrHtml = await server.renderToString(ssrApp);
  }
  async function observePacket(packet, fragments, version, state, result) {
    if (version === "3") return evaluateVue3(packet, state, result);
    result.wholeFirstRoot = {};
    result.roots = [];
    evaluateLegacy(packet, version, state, result.wholeFirstRoot);
    for (const fragment of fragments) {
      const root = { index: fragment.index, observation: {} };
      result.roots.push(root);
      evaluateLegacy(fragment.packet, version, state, root.observation);
    }
  }

  // source is the exact CLI input/output; phase is original/historical/current.
  // All packets and all four executions are persisted BEFORE contract checks.
  // The Vue2 error carrier is evaluated as the official first-root function
  // plus exact source-owned p fragments. It is never admitted as valid DOM/SSR.
  async function qualify(source, reference, phase, observed, persist = () => {}) {
    Object.assign(observed, {
      id: reference.id,
      version: reference.version,
      phase,
      identities,
      source,
      sourceSha256: sha256(source),
      packet: null,
      fragments: [],
      runtime: [],
      qualified: false,
    });
    persist();
    try {
      observed.packet = compilePacket(source, reference.version, reference.id);
      persist();
      if (reference.version !== "3")
        captureFragments(source, reference.version, observed.fragments, persist);
      const errors = observed.packet.parseErrors ?? [
        ...observed.packet.descriptorErrors,
        ...observed.packet.errors,
      ];
      observed.classification = {
        validComponent: errors.length === 0,
        sourceOwnedRootCount: reference.version === "3" ? null : observed.fragments.length,
        runtime:
          reference.version === "3"
            ? "stock Vue3 mounted DOM and SSR"
            : "official Vue2 compiler functions with declared VNode/filter observer; no Vue2 DOM/SSR claim",
      };
      persist();
      for (const control of reference.stock.states) {
        const row = { state: control.state, result: {}, error: null };
        observed.runtime.push(row);
        persist();
        try {
          await observePacket(
            observed.packet,
            observed.fragments,
            reference.version,
            control.state,
            row.result,
          );
        } catch (error) {
          row.error = failure(error);
          persist();
          throw error;
        }
        persist();
      }
      assert.ok(["original", "historical", "current"].includes(phase), "sealed history phase");
      const sealedSource =
        phase === "original"
          ? reference.input.content
          : phase === "historical"
            ? reference.historical.content
            : reference.expected;
      assert.equal(source, sealedSource, `${reference.id}: complete ${phase} source bytes`);
      assert.equal(
        observed.sourceSha256,
        phase === "original"
          ? reference.input.sha256
          : phase === "historical"
            ? reference.historical.sha256
            : reference.expectedSha256,
      );
      assert.equal(
        observed.packet.template,
        rawTemplate(source),
        "complete unmodified template content",
      );
      assert.deepEqual(
        observed.packet,
        reference.stock.packets[phase],
        `${reference.id}: full sealed ${phase} compiler packet`,
      );
      assert.equal(observed.classification.validComponent, reference.stock.validity.validComponent);
      if (reference.version !== "3") {
        assert.deepEqual(
          observed.fragments,
          reference.stock.fragments[phase],
          "exact source-owned fragments and packets",
        );
        for (const fragment of observed.fragments) assert.deepEqual(fragment.packet.errors, []);
        if (!observed.classification.validComponent) {
          assert.equal(reference.id, "sfc-vue2-filter-chain-crlf");
          assert.equal(observed.fragments.length, 2);
          assert.equal(observed.packet.errors.length, 1);
          assert.match(
            observed.packet.errors[0].msg ?? observed.packet.errors[0],
            /exactly one root element/,
          );
        }
      }
      assert.equal(observed.runtime.length, 4);
      for (let index = 0; index < observed.runtime.length; index++) {
        assert.deepEqual(
          observed.runtime[index].result,
          reference.stock.states[index].phases[phase],
          `${reference.id}: exact sealed ${phase}/${observed.runtime[index].state.id} state`,
        );
        assert.deepEqual(
          observed.runtime[index].result,
          reference.stock.states[index].phases.original,
          `${reference.id}: complete original semantic contract`,
        );
      }
      observed.qualified = true;
      persist();
      return observed;
    } catch (error) {
      observed.error = failure(error);
      persist();
      throw error;
    }
  }
  return { identities, compilePacket, observePacket, qualify };
}
