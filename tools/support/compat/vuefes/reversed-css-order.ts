export async function observeReversedCssOrder() {
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
    .filter((animation) => animation instanceof CSSTransition);
  const transitionEvidence = transitions.map((animation) => ({
    id: animation.id,
    property: animation.transitionProperty,
    target: animation.effect?.target?.outerHTML ?? null,
    playState: animation.playState,
    startTime: animation.startTime,
    currentTime: animation.currentTime,
    keyframes: animation.effect?.getKeyframes(),
    timing: animation.effect?.getTiming(),
    computedTiming: animation.effect?.getComputedTiming(),
    status: "pending",
    error: null,
  }));
  await Promise.all(
    transitions.map(async (animation, index) => {
      const evidence = transitionEvidence[index];
      if (!Number.isFinite(evidence.computedTiming?.endTime)) {
        evidence.status = "non-finite";
        return;
      }
      try {
        await animation.finished;
        evidence.status = "finished";
      } catch (error) {
        evidence.status = "cancelled";
        evidence.error = String(error.stack ?? error);
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
      owner: sheet.ownerNode.outerHTML,
      rules: [...sheet.cssRules].map((rule) => rule.cssText),
    })),
  };
}
