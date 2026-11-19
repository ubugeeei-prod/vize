<main id="root">
  <svg :width="100"><rect fill="red"></rect></svg>
  <p ref="anchor">anchored</p>
  <button @click="go">go</button>
  <a :[attr]="x">dynamic name</a>
  <span :title.trim="t">modifier</span>
  <i :class="'c'">reserved key</i>
  <em :title="msg">non-constant</em>
  <del :title="">blank value</del>
  <b :title="'kept'" :data-n="1 + 1">constant</b>
  <section><Panel /></section>
  <ul><li v-for="item in items">row</li></ul>
  <h2 class="title">{{ msg }}</h2>
  <Card size="3" />
  <Card :size="3" @close="off" />
</main>
