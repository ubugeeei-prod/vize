<ul>
  <li v-for="(item, i) in items" key="row">{{ item }}</li>
</ul>
<template v-for="({ id }, index) of rows">
  <dt v-if="id" key="term">{{ id }}</dt>
  <dd v-else>{{ index }}</dd>
</template>
<p v-for=" in mystery">fallback</p>
