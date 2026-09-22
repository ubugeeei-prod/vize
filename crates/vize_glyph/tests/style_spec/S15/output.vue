<template>
  <Layout v-slot="slotProps">
    <p>{{ slotProps.title }}</p>
    <template #header>
      <h1>Title</h1>
    </template>
  </Layout>
</template>
