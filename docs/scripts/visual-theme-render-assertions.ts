import assert from "node:assert/strict";
import type { Page } from "playwright";

/** Observe the loaded product styles and native elements, without CSS injection. */
export async function verifyVisualTheme(page: Page) {
  const observed = await page.evaluate(() => {
    const style = (selector: string, pseudo?: string) => {
      const element = document.querySelector(selector);
      if (!element) return null;
      const css = getComputedStyle(element, pseudo);
      return {
        background: css.backgroundColor,
        borderLeft: css.borderLeftWidth,
        borderRight: css.borderRightWidth,
        content: css.content,
        opacity: Number(css.opacity),
      };
    };
    const canvas = document.querySelector<HTMLCanvasElement>("#vein-canvas");
    const kinds = ["keyword", "string", "number", "type", "function", "tag"];
    return {
      theme: document.documentElement.dataset.theme,
      mobile: matchMedia("(max-width: 768px)").matches,
      entry: document.body.classList.contains("entry-page"),
      canvas: canvas && {
        width: canvas.width,
        height: canvas.height,
        clientWidth: canvas.clientWidth,
        clientHeight: canvas.clientHeight,
        premultipliedAlpha: (
          canvas.getContext("webgl2") ?? canvas.getContext("webgl")
        )?.getContextAttributes()?.premultipliedAlpha,
        ...style("#vein-canvas"),
      },
      layout: style(".layout"),
      main: style(".main"),
      sidebar: style(".sidebar"),
      navMarker: style(".nav-link.active", "::before"),
      rails: [".content blockquote", ".content blockquote.ox-callout", ".toc", ".toc-link"].map(
        (selector) => ({ selector, ...style(selector) }),
      ),
      headingBorders: [...document.querySelectorAll(".content h2")].map(
        (element) => getComputedStyle(element).borderBottomWidth,
      ),
      separators: [...document.querySelectorAll(".content hr")].map((element) => {
        const css = getComputedStyle(element);
        return {
          width: css.borderTopWidth,
          color: css.borderTopColor,
          background: getComputedStyle(document.body).backgroundColor,
        };
      }),
      palette: kinds.map((kind) => {
        const token = document.querySelector(`[data-render-palette-control] .v-code__${kind}`);
        const color = token && getComputedStyle(token).color;
        const rgb =
          color
            ?.match(/[\d.]+/g)
            ?.slice(0, 3)
            .map(Number) ?? [];
        const high = Math.max(...rgb),
          low = Math.min(...rgb),
          delta = high - low;
        const hue = !delta
          ? 0
          : high === rgb[0]
            ? (rgb[1] - rgb[2]) / delta
            : high === rgb[1]
              ? (rgb[2] - rgb[0]) / delta + 2
              : (rgb[0] - rgb[1]) / delta + 4;
        const saturation = delta ? delta / (255 - Math.abs(high + low - 255)) : 0;
        return { kind, color, hue: (hue * 60 + 360) % 360, saturation };
      }),
    };
  });
  for (const rail of observed.rails)
    if (rail.borderLeft) assert.equal(rail.borderLeft, "0px", `${page.url()}: ${rail.selector}`);
  if (observed.navMarker) assert.equal(observed.navMarker.content, "none");
  if (observed.canvas) {
    assert(observed.canvas.width > 0 && observed.canvas.height > 0, "Shader has real raster size");
    assert(observed.canvas.width >= observed.canvas.clientWidth);
    assert(observed.canvas.height >= observed.canvas.clientHeight);
    assert.equal(
      observed.canvas.premultipliedAlpha,
      true,
      "Native shader alpha matches composition",
    );
    assert(typeof observed.canvas.opacity === "number", "Shader has computed opacity");
    assert(observed.canvas.opacity >= 0.2, "Shader remains visible through the page surfaces");
    if (!observed.entry) {
      assert.equal(observed.layout?.background, "rgba(0, 0, 0, 0)");
      assert.equal(observed.main?.background, "rgba(0, 0, 0, 0)");
      if (!observed.mobile && observed.sidebar) {
        assert.equal(observed.sidebar.background, "rgba(0, 0, 0, 0)");
        assert.equal(observed.sidebar.borderRight, "0px");
      }
    }
  }
  for (const border of observed.headingBorders)
    assert.equal(border, "0px", "Content headings use spacing instead of underlines");
  const luminance = (rgb: number[]) =>
    rgb.reduce((sum, value, index) => {
      const channel = value / 255;
      const linear = channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
      return sum + linear * [0.2126, 0.7152, 0.0722][index];
    }, 0);
  for (const separator of observed.separators) {
    assert.equal(separator.width, "1px", "Native horizontal rules keep a thin line");
    const color = separator.color.match(/[\d.]+/g)?.map(Number) ?? [];
    const background =
      separator.background
        .match(/[\d.]+/g)
        ?.slice(0, 3)
        .map(Number) ?? [];
    assert(
      color.length >= 3 && background.length === 3,
      "Rendered separator colors are measurable",
    );
    assert(
      Math.max(...color.slice(0, 3)) - Math.min(...color.slice(0, 3)) <= 3,
      "Rules use a neutral color",
    );
    const alpha = color[3] ?? 1;
    const composed = color
      .slice(0, 3)
      .map((channel, index) => channel * alpha + background[index] * (1 - alpha));
    const line = luminance(composed),
      surface = luminance(background);
    const contrast = (Math.max(line, surface) + 0.05) / (Math.min(line, surface) + 0.05);
    assert(contrast > 1 && contrast < 1.3, "Decorative rules remain subtle against the page");
  }
  if (observed.palette.some(({ color }) => color)) {
    assert(
      observed.palette.every(({ color }) => color),
      "Native palette renders every token kind",
    );
    if (observed.theme === "light") {
      assert.equal(new Set(observed.palette.map(({ color }) => color)).size, 6);
      for (const token of observed.palette)
        assert(token.saturation >= 0.8, `${token.kind} remains visibly saturated`);
      for (const [index, token] of observed.palette.entries())
        for (const other of observed.palette.slice(index + 1)) {
          const difference = Math.abs(token.hue - other.hue);
          assert(Math.min(difference, 360 - difference) >= 20, `${token.kind}/${other.kind} hues`);
        }
    }
  }
  return observed;
}
