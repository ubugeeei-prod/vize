<main>
  <p v-if="ready && !failed">ready</p>
  <p v-else-if="failed || timedOut || aborted">failed</p>
  <p v-else>loading</p>
  <ul>
    <li v-for="item in items" :key="item.id" :class="item.done ? 'done' : 'todo'">
      <b v-if="item.pinned">{{ item.label ?? item.fallback ?? '-' }}</b>
    </li>
  </ul>
  <List :rows="rows">
    <template #row="{ row }">
      <i v-show="row.visible && row.enabled">{{ row.score > 9 ? (row.star ? '**' : '*') : '' }}</i>
    </template>
  </List>
  <input v-model="query" @keyup.enter="submit(); reset()" />
  <slot :name="named ? 'custom' : 'default'">fallback</slot>
</main>
