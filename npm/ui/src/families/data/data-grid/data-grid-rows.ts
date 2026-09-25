import {
  cellValue,
  columnCallbacks,
  defaultCompare,
  defaultFormat,
  formatCell,
} from "./data-grid-columns.ts";
import type { DataGridColumn, DataGridRowModel, DataGridSort } from "./data-grid-types.ts";

/** Inputs of the pure row pipeline. */
export interface DataGridRowPipeline<Row> {
  readonly rows: readonly Row[];
  readonly columns: readonly DataGridColumn<Row>[];
  readonly getRowId: (row: Row, index: number, parentId: string | null) => string;
  readonly getSubRows?: ((row: Row) => readonly Row[] | null | undefined) | undefined;
  readonly sorting: readonly DataGridSort[];
  readonly globalFilter: string;
  readonly columnFilters: Readonly<Record<string, unknown>>;
  readonly filterRow?: ((row: Row) => boolean) | undefined;
  readonly expanded: ReadonlySet<string>;
  readonly selected: ReadonlySet<string>;
}

interface TreeNode<Row> {
  readonly id: string;
  readonly row: Row;
  readonly parentId: string | null;
  readonly children: readonly TreeNode<Row>[];
}

function buildTree<Row>(
  rows: readonly Row[],
  pipeline: DataGridRowPipeline<Row>,
  parentId: string | null,
  depth: number,
): TreeNode<Row>[] {
  return rows.map((row, index) => {
    const id = pipeline.getRowId(row, index, parentId);
    const subRows = depth < 32 ? pipeline.getSubRows?.(row) : undefined;
    return {
      id,
      row,
      parentId,
      children: subRows ? buildTree(subRows, pipeline, id, depth + 1) : [],
    };
  });
}

function filterText(value: unknown): string {
  return (typeof value === "string" ? value : defaultFormat(value)).toLocaleLowerCase();
}

function matchesFilters<Row>(row: Row, pipeline: DataGridRowPipeline<Row>): boolean {
  if (pipeline.filterRow && !pipeline.filterRow(row)) return false;
  for (const column of pipeline.columns) {
    if (!Object.hasOwn(pipeline.columnFilters, column.id)) continue;
    const filterValue = pipeline.columnFilters[column.id];
    if (filterValue === undefined || filterValue === null || filterValue === "") continue;
    const callbacks = columnCallbacks(column);
    if (!callbacks) continue;
    const value = cellValue(column, row);
    const keep = callbacks.filter
      ? callbacks.filter(value, filterValue, row)
      : formatCell(column, row).toLocaleLowerCase().includes(filterText(filterValue));
    if (!keep) return false;
  }
  const query = pipeline.globalFilter.trim().toLocaleLowerCase();
  if (query.length === 0) return true;
  return pipeline.columns.some(
    (column) =>
      column.kind !== "display" && formatCell(column, row).toLocaleLowerCase().includes(query),
  );
}

/** Keep nodes that match, plus ancestors of matching descendants. */
function filterTree<Row>(
  nodes: readonly TreeNode<Row>[],
  pipeline: DataGridRowPipeline<Row>,
): TreeNode<Row>[] {
  const kept: TreeNode<Row>[] = [];
  for (const node of nodes) {
    const children = filterTree(node.children, pipeline);
    if (children.length > 0 || matchesFilters(node.row, pipeline)) kept.push({ ...node, children });
  }
  return kept;
}

function compareNodes<Row>(
  left: TreeNode<Row>,
  right: TreeNode<Row>,
  keys: readonly { readonly column: DataGridColumn<Row>; readonly sign: number }[],
): number {
  for (const { column, sign } of keys) {
    const callbacks = columnCallbacks(column);
    const a = cellValue(column, left.row);
    const b = cellValue(column, right.row);
    const result = callbacks?.compare
      ? callbacks.compare(a, b, left.row, right.row)
      : defaultCompare(a, b);
    if (result !== 0) {
      // Missing values stay last regardless of direction.
      const missing = a === null || a === undefined || b === null || b === undefined;
      return missing && !callbacks?.compare ? result : result * sign;
    }
  }
  return 0;
}

function sortTree<Row>(
  nodes: readonly TreeNode<Row>[],
  pipeline: DataGridRowPipeline<Row>,
): TreeNode<Row>[] {
  const keys = pipeline.sorting.flatMap((sort) => {
    const column = pipeline.columns.find((candidate) => candidate.id === sort.columnId);
    return column && column.kind !== "display"
      ? [{ column, sign: sort.direction === "descending" ? -1 : 1 }]
      : [];
  });
  const withChildren = nodes.map((node) => ({
    ...node,
    children: sortTree(node.children, pipeline),
  }));
  if (keys.length === 0) return withChildren;
  // Array#sort is stable, so equal keys keep source order.
  return withChildren.sort((left, right) => compareNodes(left, right, keys));
}

/** Filter, sort, and flatten rows into the visible row models. */
export function buildRowModels<Row>(pipeline: DataGridRowPipeline<Row>): DataGridRowModel<Row>[] {
  const tree = sortTree(
    filterTree(buildTree(pipeline.rows, pipeline, null, 0), pipeline),
    pipeline,
  );
  const flat: DataGridRowModel<Row>[] = [];
  const visit = (nodes: readonly TreeNode<Row>[], depth: number): void => {
    for (const node of nodes) {
      const expanded = node.children.length > 0 && pipeline.expanded.has(node.id);
      flat.push(
        Object.freeze({
          id: node.id,
          row: node.row,
          index: flat.length,
          depth,
          parentId: node.parentId,
          expandable: node.children.length > 0,
          expanded,
          selected: pipeline.selected.has(node.id),
        }),
      );
      if (expanded) visit(node.children, depth + 1);
    }
  };
  visit(tree, 0);
  return flat;
}

/** Next sorting after a user toggles `columnId`: ascending → descending → removed. */
export function toggleSorting(
  sorting: readonly DataGridSort[],
  columnId: string,
  multi: boolean,
): DataGridSort[] {
  const current = sorting.find((sort) => sort.columnId === columnId);
  const next: DataGridSort | null =
    current === undefined
      ? { columnId, direction: "ascending" }
      : current.direction === "ascending"
        ? { columnId, direction: "descending" }
        : null;
  if (!multi) return next ? [next] : [];
  const others = sorting.filter((sort) => sort.columnId !== columnId);
  if (!next) return others;
  return current
    ? sorting.map((sort) => (sort.columnId === columnId ? next : sort))
    : [...others, next];
}
