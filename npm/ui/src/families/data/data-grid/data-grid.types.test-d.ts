/** Compile-only assertions for DataGrid column inference and public contracts. */

import type {
  DataGridCellSlotProps,
  DataGridColumn,
  DataGridColumnId,
  DataGridColumnValue,
  DataGridController,
  DataGridPath,
  DataGridPathValue,
  DataGridRowModel,
  DataGridSort,
  DataGridState,
} from "./data-grid.ts";
import { createColumnHelper, DataGrid, useDataGrid } from "./data-grid.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Address {
  readonly city: string;
  readonly geo: { readonly lat: number; readonly lng: number };
}

interface User {
  readonly id: number;
  readonly name: string;
  readonly email: string | null;
  readonly born: Date;
  readonly tags: readonly string[];
  readonly address: Address;
  readonly manager?: { readonly name: string };
  readonly children?: readonly User[];
}

// --- Paths -----------------------------------------------------------------

type UserPath = DataGridPath<User>;
type _TopLevelKeys = Expect<
  Equal<Extract<UserPath, "id" | "name" | "email">, "email" | "id" | "name">
>;
type _NestedKeys = Expect<
  Equal<
    Extract<UserPath, `address.${string}`>,
    "address.city" | "address.geo" | "address.geo.lat" | "address.geo.lng"
  >
>;
type _OptionalNested = Expect<Equal<Extract<UserPath, `manager.${string}`>, "manager.name">>;
type _ArraysAreLeaves = Expect<Equal<Extract<UserPath, `tags.${string}`>, never>>;
type _DatesAreLeaves = Expect<Equal<Extract<UserPath, `born.${string}`>, never>>;
type _DepthLimited = Expect<Equal<Extract<UserPath, `children.${string}`>, never>>;

type _ValueOfString = Expect<Equal<DataGridPathValue<User, "name">, string>>;
type _ValueOfNullable = Expect<Equal<DataGridPathValue<User, "email">, string | null>>;
type _ValueOfNested = Expect<Equal<DataGridPathValue<User, "address.geo.lat">, number>>;
type _ValueThroughOptional = Expect<
  Equal<DataGridPathValue<User, "manager.name">, string | undefined>
>;
type _ValueOfDate = Expect<Equal<DataGridPathValue<User, "born">, Date>>;

// --- Column helper inference ---------------------------------------------------

const column = createColumnHelper<User>();
const nameColumn = column.accessor("name", {
  header: "Name",
  compare: (left, right) => {
    type _Left = Expect<Equal<typeof left, string>>;
    return left.localeCompare(right);
  },
  format: (value) => value.toUpperCase(),
  parse: (input) => input.trim(),
  validate: (value) => (value.length === 0 ? "Required" : null),
});
const cityColumn = column.accessor("address.city", { id: "city", header: "City" });
const latColumn = column.accessor("address.geo.lat", {
  filter: (value, filterValue) => {
    type _Value = Expect<Equal<typeof value, number>>;
    return typeof filterValue === "number" && value >= filterValue;
  },
});
const ageColumn = column.computed("age", (user) => 2026 - user.born.getFullYear(), {
  compare: (left, right) => {
    type _Left = Expect<Equal<typeof left, number>>;
    return left - right;
  },
});
const actionsColumn = column.display("actions", { width: 60, resizable: false });

type _NameId = Expect<Equal<DataGridColumnId<typeof nameColumn>, "name">>;
type _CityId = Expect<Equal<DataGridColumnId<typeof cityColumn>, "city">>;
type _AgeId = Expect<Equal<DataGridColumnId<typeof ageColumn>, "age">>;
type _ActionsId = Expect<Equal<DataGridColumnId<typeof actionsColumn>, "actions">>;
type _NameValue = Expect<Equal<DataGridColumnValue<User, typeof nameColumn>, string>>;
type _CityValue = Expect<Equal<DataGridColumnValue<User, typeof cityColumn>, string>>;
type _LatValue = Expect<Equal<DataGridColumnValue<User, typeof latColumn>, number>>;
type _AgeValue = Expect<Equal<DataGridColumnValue<User, typeof ageColumn>, number>>;
type _ActionsValue = Expect<Equal<DataGridColumnValue<User, typeof actionsColumn>, undefined>>;

const columns = [nameColumn, cityColumn, latColumn, ageColumn, actionsColumn];
type Columns = (typeof columns)[number];
const _assignable: readonly DataGridColumn<User>[] = columns;

// --- Discriminated cell slot props -------------------------------------------

declare const cell: DataGridCellSlotProps<User, Columns>;
type _SlotIds = Expect<
  Equal<typeof cell.columnId, "actions" | "address.geo.lat" | "age" | "city" | "name">
>;
if (cell.columnId === "age") {
  type _Narrowed = Expect<Equal<typeof cell.value, number>>;
}
if (cell.columnId === "name") {
  type _Narrowed = Expect<Equal<typeof cell.value, string>>;
}
if (cell.columnId === "actions") {
  type _Narrowed = Expect<Equal<typeof cell.value, undefined>>;
}
type _SlotRow = Expect<Equal<typeof cell.row, User>>;
type _SlotRowModel = Expect<Equal<typeof cell.rowModel, DataGridRowModel<User>>>;

// --- Composable ----------------------------------------------------------------

declare const controller: DataGridController<User, Columns>;
type _Rows = Expect<Equal<(typeof controller.rowModels.value)[number]["row"], User>>;
type _ColumnModel = Expect<
  Equal<(typeof controller.columnModels.value)[number]["column"], Columns>
>;
type _Sorting = Expect<Equal<DataGridState["sorting"], readonly DataGridSort[]>>;
controller.toggleSort("name", true);
controller.select("1", "range");
controller.setState("columnSizing", { name: 200 });

function setup(): void {
  const grid = useDataGrid({ rows: [] as readonly User[], columns });
  type _Inferred = Expect<Equal<typeof grid, DataGridController<User, Columns>>>;
}

// --- SFC props ----------------------------------------------------------------

type GridProps = Parameters<typeof DataGrid<User, Columns>>[0];
const gridProps: GridProps = {
  rows: [],
  columns,
  getRowId: (user) => String(user.id),
  getSubRows: (user) => user.children,
  selectionMode: "multiple",
  sorting: [{ columnId: "age", direction: "ascending" }],
  "onCell-edit": (event) => event.row.name,
};
// @ts-expect-error rows must match the column row type.
const badGridRows: GridProps = { rows: [{ id: "x" }], columns };
// @ts-expect-error selection mode is a closed union.
const badGridMode: GridProps = { rows: [], columns, selectionMode: "range" };

// --- Negative cases --------------------------------------------------------------

// @ts-expect-error unknown accessor paths are rejected.
column.accessor("address.zip");
// @ts-expect-error array members are not traversed.
column.accessor("tags.length");
column.accessor("name", {
  // @ts-expect-error compare receives the column's value type.
  compare: (left: number, right: number) => left - right,
});
// @ts-expect-error format must accept the computed value type (Date), not string.
column.computed("age", (user) => user.born, { format: (value: string) => value });
// @ts-expect-error display columns carry no value callbacks.
column.display("x", { compare: () => 0 });
// @ts-expect-error setState keys are checked against their slice type.
controller.setState("sorting", [{ columnId: "name", direction: "up" }]);
// @ts-expect-error selection intents are a closed union.
controller.select("1", "add");
const _wrongRow: readonly DataGridColumn<User>[] = [
  // @ts-expect-error the grid rejects columns built for another row type.
  createColumnHelper<{ readonly x: Date }>().accessor("x"),
];

void [badGridMode, badGridRows, gridProps];
void setup;
void _assignable;
void _wrongRow;
