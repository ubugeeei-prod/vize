// P4-16 fixtures, kept as strings so the repository's Vue lint lane never
// reads them as components.

/** Nested loops (the outer index as an inner key), a plain index key, and
 * a shadowing loop variable named `index` that is a value, not an index. */
export const TODO_LIST = `<template>
  <table>
    <tr v-for="(row, i) in rows" :key="row.id">
      <td v-for="(cell, j) in row.cells" :key="i">{{ cell }}</td>
    </tr>
  </table>
  <ul>
    <li v-for="(todo, index) in todos" :key="index">{{ todo.title }}</li>
    <li v-for="(item, index) in items" :key="item.id">
      <span v-for="index in item.tags" :key="index">{{ index }}</span>
    </li>
  </ul>
</template>
`;
