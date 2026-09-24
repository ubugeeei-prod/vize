/**
 * Minimal, dependency-free DOM stand-ins for composable tests.
 *
 * The package runs its suite on plain Node (`node --test`), so browser APIs
 * are modelled with `EventTarget` subclasses that implement exactly the
 * members the composables read. Casts to platform types are confined to the
 * `as*` helpers in this file.
 */

/** Mutable rectangle backing {@link FakeElement.getBoundingClientRect}. */
export interface FakeRect {
  /** Left edge. */
  x: number;
  /** Top edge. */
  y: number;
  /** Width. */
  width: number;
  /** Height. */
  height: number;
}

/** Document stand-in tracking focus, pointer lock, and selection. */
export class FakeDocument extends EventTarget {
  /** Nine is the platform `DOCUMENT_NODE` value. */
  readonly nodeType = 9;
  /** Focused element. */
  activeElement: FakeElement | null = null;
  /** Element holding the pointer lock. */
  pointerLockElement: FakeElement | null = null;
  /** Value returned by `getSelection`. */
  selection: FakeSelection | null = null;
  /** Root element used for window scroll metrics. */
  readonly documentElement: FakeElement;

  constructor() {
    super();
    this.documentElement = new FakeElement(this);
  }

  /** Release the pointer lock and notify listeners. */
  exitPointerLock(): void {
    this.pointerLockElement = null;
    this.dispatchEvent(new Event("pointerlockchange"));
  }

  /** Current selection stand-in. */
  getSelection(): Selection | null {
    return this.selection ? asSelection(this.selection) : null;
  }

  /** Create an element owned by this document. */
  createElement(): FakeElement {
    return new FakeElement(this);
  }
}

/** Element stand-in with geometry, scrolling, focus, and a child list. */
export class FakeElement extends EventTarget {
  /** One is the platform `ELEMENT_NODE` value. */
  readonly nodeType = 1;
  /** Owning document. */
  readonly ownerDocument: FakeDocument;
  /** Child elements for `contains`. */
  readonly children: FakeElement[] = [];
  /** Backing rectangle. */
  rect: FakeRect = { x: 0, y: 0, width: 0, height: 0 };
  /** Vertical scroll offset. */
  scrollTop = 0;
  /** Horizontal scroll offset. */
  scrollLeft = 0;
  /** Scrollable width. */
  scrollWidth = 0;
  /** Scrollable height. */
  scrollHeight = 0;
  /** Visible width. */
  clientWidth = 0;
  /** Visible height. */
  clientHeight = 0;
  /** Selectors for which `matches` returns true. */
  readonly matching = new Set<string>();
  /** Inline style declarations written through `style.setProperty`. */
  readonly styles = new Map<string, string>();
  /** Inline style stand-in. */
  readonly style = {
    setProperty: (name: string, value: string): void => {
      this.styles.set(name, value);
    },
    getPropertyValue: (name: string): string => this.styles.get(name) ?? "",
    removeProperty: (name: string): string => {
      const previous = this.styles.get(name) ?? "";
      this.styles.delete(name);
      return previous;
    },
    get overflow(): string {
      return owner(this).styles.get("overflow") ?? "";
    },
  };
  /** Upper-case tag name. */
  tagName = "DIV";
  /** Parent registered through {@link appendChild}. */
  parentElement: FakeElement | null = null;
  /** Attribute storage. */
  readonly attributes = new Map<string, string>();
  /** Class list stand-in. */
  readonly classList = new FakeClassList();
  /** Pointer ids currently captured. */
  readonly captured = new Set<number>();
  /** Whether `requestPointerLock` should fail. */
  refusePointerLock = false;
  /** Recorded `scrollTo` calls. */
  readonly scrollCalls: ScrollToOptions[] = [];
  /** Open shadow root stand-in. */
  shadowRoot: { activeElement: FakeElement | null } | null = null;

  constructor(ownerDocument: FakeDocument) {
    super();
    this.ownerDocument = ownerDocument;
    styleOwners.set(this.style, this);
  }

  /** Read an attribute. */
  getAttribute(name: string): string | null {
    return this.attributes.get(name) ?? null;
  }

  /** Write an attribute. */
  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value);
  }

  /** Append a child element. */
  append(child: FakeElement): void {
    this.children.push(child);
  }

  /** Whether `node` is this element or a descendant. */
  contains(node: unknown): boolean {
    if (node === this) return true;
    return this.children.some((child) => child.contains(node));
  }

  /** Viewport rectangle derived from {@link FakeElement.rect}. */
  getBoundingClientRect(): DOMRect {
    return createRect(this.rect);
  }

  /** Whether the element matches a selector registered in {@link FakeElement.matching}. */
  matches(selector: string): boolean {
    return this.matching.has(selector);
  }

  /** Focus the element and dispatch `blur`/`focusout`/`focus`/`focusin`. */
  focus(): void {
    const previous = this.ownerDocument.activeElement;
    if (previous === this) return;
    this.ownerDocument.activeElement = this;
    if (previous) {
      previous.dispatchEvent(new Event("blur"));
      previous.bubble(eventWith("focusout", { relatedTarget: this }));
    }
    this.dispatchEvent(new Event("focus"));
    this.bubble(eventWith("focusin", { relatedTarget: previous }));
    this.ownerDocument.dispatchEvent(new Event("focus"));
  }

  /** Blur the element and dispatch `blur`/`focusout`. */
  blur(): void {
    if (this.ownerDocument.activeElement !== this) return;
    this.ownerDocument.activeElement = null;
    this.dispatchEvent(new Event("blur"));
    this.bubble(eventWith("focusout", { relatedTarget: null }));
    this.ownerDocument.dispatchEvent(new Event("blur"));
  }

  /** Scroll and dispatch `scroll`. */
  scrollTo(options: ScrollToOptions): void {
    this.scrollCalls.push(options);
    if (options.left !== undefined) this.scrollLeft = options.left;
    if (options.top !== undefined) this.scrollTop = options.top;
    this.dispatchEvent(new Event("scroll"));
  }

  /** Record a pointer capture. */
  setPointerCapture(pointerId: number): void {
    this.captured.add(pointerId);
  }

  /** Release a pointer capture. */
  releasePointerCapture(pointerId: number): void {
    this.captured.delete(pointerId);
  }

  /** Acquire (or refuse) the pointer lock asynchronously, like browsers do. */
  requestPointerLock(): Promise<void> {
    return Promise.resolve().then(() => {
      if (this.refusePointerLock) {
        this.ownerDocument.dispatchEvent(new Event("pointerlockerror"));
        return;
      }
      this.ownerDocument.pointerLockElement = this;
      this.ownerDocument.dispatchEvent(new Event("pointerlockchange"));
    });
  }

  /** Dispatch on this element and every registered ancestor. */
  bubble(event: Event): void {
    this.dispatchEvent(event);
    for (const ancestor of ancestors.get(this) ?? []) ancestor.dispatchEvent(event);
  }
}

const ancestors = new WeakMap<FakeElement, FakeElement[]>();
const styleOwners = new WeakMap<object, FakeElement>();

function owner(style: object): FakeElement {
  const element = styleOwners.get(style);
  if (!element) throw new Error("style stand-in without an owner");
  return element;
}

/** `DOMTokenList` stand-in. */
export class FakeClassList {
  /** Current tokens. */
  readonly tokens = new Set<string>();

  /** Add a token. */
  add(token: string): void {
    this.tokens.add(token);
  }

  /** Remove a token. */
  remove(token: string): void {
    this.tokens.delete(token);
  }

  /** Whether a token is present. */
  contains(token: string): boolean {
    return this.tokens.has(token);
  }
}

/**
 * Append `child` to `parent` and register the ancestry used by bubbling
 * focus events.
 */
export function appendChild(parent: FakeElement, child: FakeElement): void {
  parent.append(child);
  child.parentElement = parent;
  ancestors.set(child, [parent, ...(ancestors.get(parent) ?? [])]);
}

/** Selection stand-in. */
export class FakeSelection {
  /** Selected text. */
  text = "";
  /** Selected ranges. */
  ranges: FakeRect[] = [];

  /** Number of ranges. */
  get rangeCount(): number {
    return this.ranges.length;
  }

  /** Range stand-in whose rectangle is the stored {@link FakeRect}. */
  getRangeAt(index: number): Range {
    const rect = this.ranges[index] ?? { x: 0, y: 0, width: 0, height: 0 };
    return asRange({ getBoundingClientRect: () => createRect(rect) });
  }

  /** Selected text. */
  toString(): string {
    return this.text;
  }
}

/** Window stand-in with sizing, scrolling, and a document. */
export class FakeWindow extends EventTarget {
  innerWidth = 1024;
  innerHeight = 768;
  outerWidth = 1040;
  outerHeight = 800;
  scrollX = 0;
  scrollY = 0;
  readonly document = new FakeDocument();
  readonly scrollCalls: ScrollToOptions[] = [];

  /** Scroll the document and dispatch `scroll`. */
  scrollTo(options: ScrollToOptions): void {
    this.scrollCalls.push(options);
    if (options.left !== undefined) this.scrollX = options.left;
    if (options.top !== undefined) this.scrollY = options.top;
    this.dispatchEvent(new Event("scroll"));
  }
}

/** Build an `Event` carrying extra readonly-looking fields (e.g. `key`, `clientX`). */
export function eventWith(type: string, fields: Readonly<Record<string, unknown>>): Event {
  const event = new Event(type, { cancelable: true });
  for (const [name, value] of Object.entries(fields)) {
    Object.defineProperty(event, name, { value, enumerable: true });
  }
  return event;
}

/** Build a keyboard-like event. */
export function keyEvent(type: "keydown" | "keyup", key: string, code = ""): Event {
  return eventWith(type, { key, code });
}

/** Build a pointer-like event. */
export function pointerEvent(
  type: string,
  fields: {
    readonly pointerId?: number;
    readonly pointerType?: string;
    readonly clientX?: number;
    readonly clientY?: number;
    readonly button?: number;
  } = {},
): Event {
  return eventWith(type, {
    pointerId: 1,
    pointerType: "mouse",
    clientX: 0,
    clientY: 0,
    pageX: fields.clientX ?? 0,
    pageY: fields.clientY ?? 0,
    screenX: fields.clientX ?? 0,
    screenY: fields.clientY ?? 0,
    button: 0,
    pressure: 0.5,
    tiltX: 0,
    tiltY: 0,
    width: 1,
    height: 1,
    twist: 0,
    ...fields,
  });
}

/** Build a touch-like event with one touch point per coordinate. */
export function touchEvent(type: string, points: readonly { x: number; y: number }[]): Event {
  const touches = points.map((point) => ({
    clientX: point.x,
    clientY: point.y,
    pageX: point.x,
    pageY: point.y,
    screenX: point.x,
    screenY: point.y,
  }));
  return eventWith(type, {
    touches: type === "touchend" ? [] : touches,
    changedTouches: touches,
  });
}

/** Constructor stand-in that records instances for observers (resize, intersection, mutation). */
export class FakeObserver<Callback> {
  /** All constructed observers, newest last. */
  static readonly instances: FakeObserver<unknown>[] = [];
  /** Callback passed to the constructor. */
  readonly callback: Callback;
  /** Constructor options (intersection observers). */
  readonly init: unknown;
  /** Currently observed targets with their options. */
  readonly observed = new Map<unknown, unknown>();
  /** Whether `disconnect` was called. */
  disconnected = false;
  /** Records returned by `takeRecords`. */
  pending: unknown[] = [];

  constructor(callback: Callback, init?: unknown) {
    this.callback = callback;
    this.init = init;
    FakeObserver.instances.push(this);
  }

  /** Start observing a target. */
  observe(target: unknown, options?: unknown): void {
    this.observed.set(target, options);
  }

  /** Stop observing a target. */
  unobserve(target: unknown): void {
    this.observed.delete(target);
  }

  /** Stop observing everything. */
  disconnect(): void {
    this.disconnected = true;
    this.observed.clear();
  }

  /** Drain pending records. */
  takeRecords(): unknown[] {
    const records = this.pending;
    this.pending = [];
    return records;
  }
}

/** Resolve the newest live fake observer. */
export function latestObserver(): FakeObserver<unknown> {
  const observer = FakeObserver.instances.at(-1);
  if (!observer) throw new Error("no observer was constructed");
  return observer;
}

/** Invoke a fake observer's callback with arbitrary entries. */
export function trigger(observer: FakeObserver<unknown>, entries: readonly unknown[]): void {
  const callback = observer.callback;
  if (typeof callback !== "function") throw new Error("observer callback is not callable");
  callback(entries, observer);
}

/** Host exposing the fake observer as `ResizeObserver`. */
export function resizeObserverHost(): { readonly ResizeObserver: typeof ResizeObserver } {
  return { ResizeObserver: asConstructor<typeof ResizeObserver>(FakeObserver) };
}

/** Host exposing the fake observer as `IntersectionObserver`. */
export function intersectionObserverHost(): {
  readonly IntersectionObserver: typeof IntersectionObserver;
} {
  return { IntersectionObserver: asConstructor<typeof IntersectionObserver>(FakeObserver) };
}

/** Host exposing the fake observer as `MutationObserver`. */
export function mutationObserverHost(): { readonly MutationObserver: typeof MutationObserver } {
  return { MutationObserver: asConstructor<typeof MutationObserver>(FakeObserver) };
}

/** Treat a fake element as a platform element. */
export function asElement(element: FakeElement): Element {
  return element as unknown as Element;
}

/** Treat a fake window as a platform window. */
export function asWindow(value: FakeWindow): Window {
  return value as unknown as Window;
}

/** Treat a fake document as a platform document. */
export function asDocument(value: FakeDocument): Document {
  return value as unknown as Document;
}

function asSelection(value: FakeSelection): Selection {
  return value as unknown as Selection;
}

function asRange(value: { getBoundingClientRect: () => DOMRect }): Range {
  return value as unknown as Range;
}

function asConstructor<Constructor>(value: unknown): Constructor {
  return value as Constructor;
}

function createRect(rect: FakeRect): DOMRect {
  const value = {
    x: rect.x,
    y: rect.y,
    width: rect.width,
    height: rect.height,
    top: rect.y,
    left: rect.x,
    right: rect.x + rect.width,
    bottom: rect.y + rect.height,
    toJSON: () => ({ ...rect }),
  };
  return value;
}

/** Forget every recorded fake observer. */
export function resetObservers(): void {
  FakeObserver.instances.length = 0;
}
