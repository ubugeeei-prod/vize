/**
 * Test-only layout model for carousel tests. happy-dom performs no layout, so
 * this installs a deterministic scroll track on the element prototypes: every
 * viewport is `size` pixels wide (or tall) and slide `n` starts at `n * size`
 * minus the viewport's scroll position. `scrollTo` calls are recorded.
 */

/** One recorded `scrollTo` call. */
export interface RecordedScroll {
  readonly target: Element;
  readonly left: number | undefined;
  readonly top: number | undefined;
  readonly behavior: ScrollBehavior | undefined;
}

/** Handle returned by {@link installCarouselGeometry}. */
export interface CarouselGeometry {
  readonly scrolls: RecordedScroll[];
  readonly position: (target: Element) => number;
  readonly setPosition: (target: Element, position: number) => void;
  readonly restore: () => void;
}

const VIEWPORT = '[data-vize-ui="carousel-viewport"]';

/** Install the layout model; call `restore()` in a `finally` block. */
export function installCarouselGeometry(size = 100, slideSize = size): CarouselGeometry {
  const positions = new WeakMap<Element, number>();
  const scrolls: RecordedScroll[] = [];
  const prototype = HTMLElement.prototype;
  const originals = new Map<PropertyKey, PropertyDescriptor | undefined>();
  const position = (target: Element): number => positions.get(target) ?? 0;
  const isViewport = (target: Element): boolean =>
    target.getAttribute("data-vize-ui") === "carousel-viewport";
  const isVertical = (target: Element): boolean =>
    target.getAttribute("data-orientation") === "vertical";
  const isRtl = (target: Element): boolean =>
    target.closest("[dir]")?.getAttribute("dir") === "rtl";
  const slideCount = (target: Element): number =>
    target.querySelectorAll('[data-vize-ui="carousel-slide"]').length;

  function define(name: PropertyKey, descriptor: PropertyDescriptor): void {
    if (!originals.has(name)) originals.set(name, Object.getOwnPropertyDescriptor(prototype, name));
    Object.defineProperty(prototype, name, { configurable: true, ...descriptor });
  }

  function original(name: PropertyKey, target: HTMLElement): unknown {
    const descriptor =
      originals.get(name) ?? Object.getOwnPropertyDescriptor(Element.prototype, name);
    if (descriptor?.get) return descriptor.get.call(target);
    return undefined;
  }

  function rect(left: number, top: number, width: number, height: number): DOMRect {
    return {
      x: left,
      y: top,
      left,
      top,
      width,
      height,
      right: left + width,
      bottom: top + height,
      toJSON: () => ({}),
    };
  }

  define("getBoundingClientRect", {
    value(this: HTMLElement): DOMRect {
      if (isViewport(this)) return rect(0, 0, size, size);
      if (this.getAttribute("data-vize-ui") === "carousel-slide") {
        const viewport = this.closest(VIEWPORT);
        const index = Number(this.getAttribute("data-index"));
        const offset = index * slideSize + (viewport === null ? 0 : -Math.abs(position(viewport)));
        if (viewport !== null && isVertical(viewport)) return rect(0, offset, size, slideSize);
        if (viewport !== null && isRtl(viewport)) {
          return rect(size - slideSize - offset, 0, slideSize, size);
        }
        return rect(offset, 0, slideSize, size);
      }
      return rect(0, 0, 0, 0);
    },
  });
  for (const name of ["scrollLeft", "scrollTop"] as const) {
    define(name, {
      get(this: HTMLElement): unknown {
        if (!isViewport(this)) return original(name, this);
        const axisMatches = (name === "scrollTop") === isVertical(this);
        return axisMatches ? position(this) : 0;
      },
      set(this: HTMLElement, value: number) {
        if (isViewport(this)) positions.set(this, value);
      },
    });
  }
  for (const name of ["clientWidth", "clientHeight"] as const) {
    define(name, {
      get(this: HTMLElement): unknown {
        return isViewport(this) ? size : original(name, this);
      },
    });
  }
  for (const name of ["scrollWidth", "scrollHeight"] as const) {
    define(name, {
      get(this: HTMLElement): unknown {
        if (!isViewport(this)) return original(name, this);
        const axisMatches = (name === "scrollHeight") === isVertical(this);
        return axisMatches ? Math.max(size, slideCount(this) * slideSize) : size;
      },
    });
  }
  define("scrollTo", {
    value(this: HTMLElement, options?: ScrollToOptions | number): void {
      if (typeof options !== "object") return;
      scrolls.push({
        target: this,
        left: options.left,
        top: options.top,
        behavior: options.behavior,
      });
      const next = isVertical(this) ? options.top : options.left;
      if (next !== undefined) positions.set(this, next);
    },
  });

  return {
    scrolls,
    position,
    setPosition: (target, value) => positions.set(target, value),
    restore() {
      for (const [name, descriptor] of originals) {
        if (descriptor) Object.defineProperty(prototype, name, descriptor);
        else Reflect.deleteProperty(prototype, name);
      }
    },
  };
}
