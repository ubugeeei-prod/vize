<article v-if="kind === 'story'" key="story">
  <h1>{{ title }}</h1>
</article>
<template v-else-if="kind === 'gallery'">
  <img v-for="src in images" key="img">
</template>
<aside v-else key="fallback">none</aside>
