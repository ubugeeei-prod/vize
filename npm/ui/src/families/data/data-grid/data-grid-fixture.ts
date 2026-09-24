import { createColumnHelper } from "./data-grid-columns.ts";

/** Row type shared by the DataGrid tests. */
export interface Person {
  readonly id: string;
  readonly name: string;
  readonly age: number;
  readonly team: { readonly name: string } | null;
  readonly reports?: readonly Person[];
}

export const people: readonly Person[] = [
  { id: "ada", name: "Ada", age: 36, team: { name: "Core" } },
  { id: "grace", name: "Grace", age: 45, team: { name: "Compilers" } },
  { id: "alan", name: "Alan", age: 41, team: null },
  { id: "linus", name: "Linus", age: 36, team: { name: "Core" } },
];

export const tree: readonly Person[] = [
  {
    id: "ceo",
    name: "Chief",
    age: 60,
    team: null,
    reports: [
      { id: "cto", name: "Tech", age: 50, team: { name: "Core" } },
      { id: "cfo", name: "Money", age: 55, team: null },
    ],
  },
  { id: "solo", name: "Solo", age: 30, team: null },
];

const column = createColumnHelper<Person>();

export const personColumns = [
  column.accessor("name", {
    header: "Name",
    editable: true,
    validate: (value) => (value.trim().length === 0 ? "Name is required" : null),
  }),
  column.accessor("age", {
    header: "Age",
    parse: (input) => Number(input),
    editable: (row) => row.age < 60,
    filter: (value, minimum) => typeof minimum === "number" && value >= minimum,
  }),
  column.accessor("team.name", { id: "team", header: "Team" }),
  column.display("actions", { header: "Actions", resizable: false, hideable: false }),
];

export type PersonColumn = (typeof personColumns)[number];
