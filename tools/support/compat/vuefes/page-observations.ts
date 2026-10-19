// Each function runs inside the page through Playwright's `page.evaluate`, so
// it must stay self-contained (no imports or module-scope helpers).

export interface FixtureResult {
  readonly complete: boolean;
  readonly warnings: readonly string[];
}
declare global {
  interface Window {
    __vuefesFixture?: FixtureResult;
  }
}

export interface BrowserObservation {
  readonly html: string;
  readonly innerHtml: string;
  readonly result: FixtureResult;
  readonly headings: readonly { html: string; fontSize: string; lineHeight: string }[];
  readonly sheets: readonly unknown[];
}

export interface HeadingStyle {
  readonly html: string;
  readonly fontSize: string;
  readonly lineHeight: string;
}

export interface TransitionEvidence {
  readonly id: string;
  readonly property: string;
  readonly target: string | null;
  readonly playState: AnimationPlayState;
  readonly startTime: CSSNumberish | null;
  readonly currentTime: CSSNumberish | null;
  readonly keyframes: ComputedKeyframe[] | undefined;
  readonly timing: EffectTiming | undefined;
  readonly computedTiming: ComputedEffectTiming | undefined;
  status: "pending" | "non-finite" | "finished" | "cancelled";
  error: string | null;
}

export interface ReversedCssOrderObservation {
  readonly intermediateHeadings: readonly (HeadingStyle & { height: number })[];
  readonly transitionEvidence: readonly TransitionEvidence[];
  readonly html: string;
  readonly headings: readonly HeadingStyle[];
  readonly sheets: readonly { href: string | null; owner: string | null; rules: string[] }[];
}

export async function observeReversedCssOrder(): Promise<ReversedCssOrderObservation> {
  const globals = [...document.head.querySelectorAll("style[data-vite-dev-id]")].filter((node) =>
    (node.getAttribute("data-vite-dev-id") ?? "").includes("/assets/styles/main.css"),
  );
  if (!globals.length) throw Error("original global stylesheet missing");
  for (const node of globals) document.head.append(node);
  // Read the actual layout to start the original finite transitions.
  // The full main sheet gives hovered elements a 100ms font-size
  // transition; two animation frames can still observe 19.666px.
  const intermediateHeadings = [...document.querySelectorAll("h2")].map((heading) => ({
    html: heading.outerHTML,
    fontSize: getComputedStyle(heading).fontSize,
    lineHeight: getComputedStyle(heading).lineHeight,
    height: heading.getBoundingClientRect().height,
  }));
  const transitions = document
    .getAnimations()
    .filter((animation): animation is CSSTransition => animation instanceof CSSTransition);
  const transitionEvidence = transitions.map((animation): TransitionEvidence => {
    const effect = animation.effect instanceof KeyframeEffect ? animation.effect : null;
    return {
      id: animation.id,
      property: animation.transitionProperty,
      target: effect?.target instanceof Element ? effect.target.outerHTML : null,
      playState: animation.playState,
      startTime: animation.startTime,
      currentTime: animation.currentTime,
      keyframes: effect?.getKeyframes(),
      timing: animation.effect?.getTiming(),
      computedTiming: animation.effect?.getComputedTiming(),
      status: "pending",
      error: null,
    };
  });
  await Promise.all(
    transitions.map(async (animation, index) => {
      const evidence = transitionEvidence[index];
      if (!evidence) return;
      if (!Number.isFinite(evidence.computedTiming?.endTime)) {
        evidence.status = "non-finite";
        return;
      }
      try {
        await animation.finished;
        evidence.status = "finished";
      } catch (error) {
        evidence.status = "cancelled";
        evidence.error = error instanceof Error ? (error.stack ?? String(error)) : String(error);
      }
    }),
  );
  return {
    intermediateHeadings,
    transitionEvidence,
    html: document.documentElement.outerHTML,
    headings: [...document.querySelectorAll("h2")].map((heading) => ({
      html: heading.outerHTML,
      fontSize: getComputedStyle(heading).fontSize,
      lineHeight: getComputedStyle(heading).lineHeight,
    })),
    sheets: [...document.styleSheets].map((sheet) => ({
      href: sheet.href,
      owner: sheet.ownerNode instanceof Element ? sheet.ownerNode.outerHTML : null,
      rules: [...sheet.cssRules].map((rule) => rule.cssText),
    })),
  };
}

/** The complete DOM, fixture result, heading styles and CSSOM after mount. */
export function observeFixturePage(): BrowserObservation {
  return {
    html: document.documentElement.outerHTML,
    innerHtml: document.querySelector("#app")?.innerHTML ?? "",
    result: window.__vuefesFixture ?? { complete: false, warnings: [] },
    headings: [...document.querySelectorAll("h2")].map((heading) => ({
      html: heading.outerHTML,
      fontSize: getComputedStyle(heading).fontSize,
      lineHeight: getComputedStyle(heading).lineHeight,
      height: heading.getBoundingClientRect().height,
    })),
    sheets: [...document.styleSheets].map((sheet) => ({
      href: sheet.href,
      owner: sheet.ownerNode instanceof Element ? sheet.ownerNode.outerHTML : null,
      rules: [...sheet.cssRules].map((rule) => rule.cssText),
    })),
  };
}
