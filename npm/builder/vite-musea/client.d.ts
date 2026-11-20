type MuseaArtStatus = "draft" | "ready" | "deprecated";

interface MuseaArtOptions {
  title?: string;
  description?: string;
  category?: string;
  tags?: string[];
  status?: MuseaArtStatus;
  order?: number;
  actionEvents?: string[];
  parameters?: Record<string, unknown>;
}

type MuseaComponentProps<TComponent> = TComponent extends abstract new (...args: never[]) => {
  $props: infer TProps;
}
  ? TProps
  : TComponent extends (props: infer TProps, ...args: never[]) => unknown
    ? TProps
    : Record<string, unknown>;

type MuseaAttrs = Readonly<Record<string, unknown>>;

type MuseaComponentAttrs<TComponent> = TComponent extends abstract new (...args: never[]) => {
  $attrs: infer TAttrs;
}
  ? TAttrs
  : MuseaAttrs;

type MuseaComponent =
  | (abstract new (...args: never[]) => { $props: unknown })
  | ((...args: never[]) => unknown);

interface MuseaCanvas {
  readonly element: HTMLElement;
  getByRole(role: string, options?: { name?: string | RegExp }): Element;
  getAllByRole(role: string, options?: { name?: string | RegExp }): Element[];
  queryByRole(role: string, options?: { name?: string | RegExp }): Element | null;
  getByText(text: string | RegExp): Element;
  getAllByText(text: string | RegExp): Element[];
  queryByText(text: string | RegExp): Element | null;
  getByLabelText(text: string | RegExp): Element;
  queryByLabelText(text: string | RegExp): Element | null;
  querySelector<E extends Element = Element>(selector: string): E | null;
  querySelectorAll<E extends Element = Element>(selector: string): E[];
}

interface MuseaUserEvent {
  click(element: Element): Promise<void>;
  dblClick(element: Element): Promise<void>;
  clear(element: Element): Promise<void>;
  type(element: Element, text: string): Promise<void>;
  selectOptions(element: HTMLSelectElement, values: string | string[]): Promise<void>;
}

interface MuseaExpectation<T = unknown> {
  readonly not: MuseaExpectation<T>;
  toBe(expected: unknown): void;
  toEqual(expected: unknown): void;
  toBeTruthy(): void;
  toBeFalsy(): void;
  toContain(expected: unknown): void;
  toBeInTheDocument(): void;
  toBeVisible(): void;
  toHaveTextContent(expected: string | RegExp): void;
  toHaveValue(expected: unknown): void;
  toBeChecked(): void;
  toHaveAttribute(name: string, value?: unknown): void;
  toHaveClass(...names: string[]): void;
}

interface MuseaPlayContext {
  readonly canvas: MuseaCanvas;
  readonly canvasElement: HTMLElement;
  readonly user: MuseaUserEvent;
  readonly userEvent: MuseaUserEvent;
  readonly expect: <T>(actual: T) => MuseaExpectation<T>;
  step<T>(name: string, run: () => T | Promise<T>): Promise<T>;
}

type MuseaPlay = (context: MuseaPlayContext) => void | Promise<void>;

declare function definePlay(play: MuseaPlay): MuseaPlay;

/** Optional gallery metadata. The target component is inferred from variant templates. */
declare function defineArtMeta(options: MuseaArtOptions): void;

/** Explicit target escape hatch for Art files with multiple component candidates. */
declare function defineArt<const TComponent extends MuseaComponent>(
  component: TComponent,
  options?: MuseaArtOptions,
): void;

declare module "*.art.vue" {
  const component: import("vue").DefineComponent<{}, {}, any>;
  export default component;
}
