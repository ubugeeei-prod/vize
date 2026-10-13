<script setup>
import { ref } from 'vue'

const a = ref(1)
const b = ref(2)
const items = ref([1, 2, 3])
const text = ref('')
const pick = (value) => value
</script>

<template>
  <section :title="'&lt;b&gt;' + a" :data-and=" a &amp;&amp; b ">
    <p v-if="a &gt; 0 &amp;&amp; b &lt; 3">{{ a &lt; b ? '&#60;' : '&#x3E;' }}</p>
    <p v-else-if="a &amp;&amp; !b">{{ a &amp;&amp; b }}</p>
    <span v-show="a &gt;= b">shown</span>
    <i v-for="item in items.filter(x =&gt; x &gt; 1)" :key="item">{{ item }}</i>
    <button @click=" a = pick(b &amp; 1) " @focus="event =&gt; pick(event)">go</button>
    <input v-model="text" :placeholder="a &lt; b &amp;&amp; '&quot;q&quot;'" />
  </section>
</template>
