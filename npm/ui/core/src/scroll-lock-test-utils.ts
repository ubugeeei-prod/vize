interface WindowMetrics {
  readonly clientWidth: number;
  readonly innerWidth: number;
  readonly scrollX: number;
  readonly scrollY: number;
  readonly maxTouchPoints?: number;
  readonly platform?: string;
  readonly supportsScrollbarGutter?: boolean;
  readonly userAgent?: string;
}

export interface WindowMetricsHarness {
  readonly scrollCalls: ScrollToOptions[];
  readonly restore: () => void;
}

function replaceProperty(target: object, key: PropertyKey, value: unknown): () => void {
  const descriptor = Object.getOwnPropertyDescriptor(target, key);
  Object.defineProperty(target, key, { configurable: true, value });
  return () => {
    if (descriptor) Object.defineProperty(target, key, descriptor);
    else Reflect.deleteProperty(target, key);
  };
}

export function mockWindowMetrics(
  document: Document,
  metrics: WindowMetrics,
): WindowMetricsHarness {
  const view = document.defaultView;
  if (!view) throw new Error("A browsing-context document is required");
  const scrollCalls: ScrollToOptions[] = [];
  const restores = [
    replaceProperty(view, "innerWidth", metrics.innerWidth),
    replaceProperty(view, "scrollX", metrics.scrollX),
    replaceProperty(view, "scrollY", metrics.scrollY),
    replaceProperty(document.documentElement, "clientWidth", metrics.clientWidth),
    replaceProperty(view, "scrollTo", (options: ScrollToOptions) => scrollCalls.push(options)),
  ];
  if (metrics.supportsScrollbarGutter !== undefined) {
    restores.push(
      replaceProperty(view, "CSS", {
        supports: (property: string, value: string) =>
          property === "scrollbar-gutter" && value === "stable" && metrics.supportsScrollbarGutter,
      }),
    );
  }
  if (metrics.platform !== undefined) {
    restores.push(replaceProperty(view.navigator, "platform", metrics.platform));
  }
  if (metrics.maxTouchPoints !== undefined) {
    restores.push(replaceProperty(view.navigator, "maxTouchPoints", metrics.maxTouchPoints));
  }
  if (metrics.userAgent !== undefined) {
    restores.push(replaceProperty(view.navigator, "userAgent", metrics.userAgent));
  }
  return {
    scrollCalls,
    restore: () => {
      for (const restore of restores.reverse()) restore();
    },
  };
}

export function preserveDocumentPresentation(document: Document): () => void {
  const root = document.documentElement;
  const body = document.body;
  const rootStyle = root.getAttribute("style");
  const bodyStyle = body.getAttribute("style");
  const attribute = root.getAttribute("data-vize-scroll-locked");
  return () => {
    if (rootStyle === null) root.removeAttribute("style");
    else root.setAttribute("style", rootStyle);
    if (bodyStyle === null) body.removeAttribute("style");
    else body.setAttribute("style", bodyStyle);
    if (attribute === null) root.removeAttribute("data-vize-scroll-locked");
    else root.setAttribute("data-vize-scroll-locked", attribute);
  };
}
