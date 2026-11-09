import { createAstro as $$createAstro, createComponent as $$createComponent, render as $$render, addAttribute as $$addAttribute } from "astro/runtime/server/index.js";
import { formatTitle } from "./format.js";

const $$Astro = $$createAstro();
const $$Component = $$createComponent(($$result, $$props, $$slots) => {
  const Astro = $$result.createAstro($$Astro, $$props, $$slots);
  Astro.self = $$Component;
  const { title, items } = Astro.props;
  const count = items.length;
  const tone = count > 2 ? "busy" : "calm";
  return $$render`<section class="card"><h1${$$addAttribute(tone, "class")}>${formatTitle(title)}</h1>${count > 0 && $$render`<p>${count} items</p>`}<ul>${items.map((item) => $$render`<li>${item}</li>`)}</ul></section>`;
});

export default $$Component;
