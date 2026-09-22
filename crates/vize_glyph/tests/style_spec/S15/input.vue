<template>
  <Layout #default="slotProps">
    <p>{{ slotProps.title }}</p>
    <template v-slot:header>
      <h1>Title</h1>
    </template>
  </Layout>
</template>
