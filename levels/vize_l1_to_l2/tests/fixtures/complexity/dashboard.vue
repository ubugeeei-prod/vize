<section class="dashboard">
  <header>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <button v-if="isAdmin && !readonly" @click="openSettings">Settings</button>
  </header>
  <DataTable :rows="rows" :loading="loading || refreshing">
    <template #cell="{ row, column }">
      <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">
        {{ row.status ?? 'unknown' }}
      </span>
      <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
      <template v-else>
        <em v-for="tag in row.tags" :key="tag">{{ tag }}</em>
      </template>
    </template>
  </DataTable>
  <p v-if="!rows.length && !loading">No data</p>
  <Pagination v-model:page="page" :total="total" @change="reload" />
</section>
