import type { CurveFactory, CurveName } from "../chart-shape/chart-shape.ts";

/** Space reserved around the plot area for axes and labels, in CSS pixels. */
export interface ChartMargin {
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
  readonly left: number;
}

/** Kind of a registered series, used by legends and data tables. */
export type ChartSeriesKind = "area" | "bars" | "line" | "pie" | "points";

/** A series registered with a ChartRoot. */
export interface ChartSeriesInfo {
  /** Stable series name, also its legend label. */
  readonly name: string;

  /** Rendering kind. */
  readonly kind: ChartSeriesKind;

  /** Whether the series is currently hidden through the legend. */
  readonly hidden: boolean;
}

/** Why the active data point changed. */
export type ChartActiveReason = "keyboard" | "pointer" | "programmatic";

/** The data point that owns keyboard focus or pointer hover. */
export interface ChartActivePoint {
  /** Series name of the point. */
  readonly series: string;

  /** Index of the datum in its series data. */
  readonly index: number;

  /** Plot-area x coordinate. */
  readonly x: number;

  /** Plot-area y coordinate. */
  readonly y: number;

  /** Accessible description of the point. */
  readonly label: string;
}

/** Plot geometry shared with every chart part. */
export interface ChartDimensions {
  /** Outer SVG width. */
  readonly width: number;

  /** Outer SVG height. */
  readonly height: number;

  /** Plot area width (width minus horizontal margins). */
  readonly innerWidth: number;

  /** Plot area height (height minus vertical margins). */
  readonly innerHeight: number;

  /** Resolved margins. */
  readonly margin: ChartMargin;
}

/** State exposed to the ChartRoot slots. */
export interface ChartSlotState extends ChartDimensions {
  /** Registered series in registration order. */
  readonly series: readonly ChartSeriesInfo[];

  /** Active data point, or `null`. */
  readonly active: ChartActivePoint | null;
}

/** Public instance exposed by ChartRoot. */
export interface ChartRootExpose extends ChartDimensions {
  /** Rendered figure element. */
  readonly element: HTMLElement | null;

  /** Active data point. */
  readonly active: ChartActivePoint | null;

  /** Set or clear the active data point. */
  readonly setActive: (point: ChartActivePoint | null) => void;

  /** Show or hide a series by name. */
  readonly toggleSeries: (name: string, hidden?: boolean) => boolean;
}

/** Scale contract accepted by ChartAxis and ChartGrid. */
export interface ChartAxisScale<Tick> {
  /** Map a tick value to a coordinate. */
  (value: Tick): number;

  /** Output extent. */
  readonly range: readonly number[];

  /** Tick values; band and point scales return every category. */
  readonly ticks: (count?: number) => Tick[];

  /** Band width, so band ticks can be centered. */
  readonly bandwidth?: number;
}

/** Placement of an axis relative to the plot area. */
export type ChartAxisOrientation = "bottom" | "left" | "right" | "top";

/** One rendered axis tick. */
export interface ChartAxisTick<Tick> {
  /** Tick value. */
  readonly value: Tick;

  /** Coordinate along the axis. */
  readonly offset: number;

  /** Formatted label. */
  readonly label: string;

  /** Zero-based position. */
  readonly index: number;
}

/** Curve option accepted by line and area series. */
export type ChartCurve = CurveName | CurveFactory;

/** A column of the accessible data table fallback. */
export interface ChartTableColumn<T> {
  /** Stable column key. */
  readonly key: string;

  /** Column header text. */
  readonly header: string;

  /** Cell value for a row. */
  readonly value: (datum: T, index: number) => string | number | Date | null | undefined;

  /**
   * Cell formatter. Defaults to locale formatting for numbers and ISO dates.
   *
   * @default undefined
   */
  readonly format?: (value: string | number | Date | null | undefined, datum: T) => string;

  /**
   * Render this column's cells as row headers (`<th scope="row">`).
   *
   * @default false
   */
  readonly rowHeader?: boolean;
}
