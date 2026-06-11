#!/usr/bin/env node
/**
 * Generate SFC files for benchmarking
 * Usage: node generate.mjs [count]
 */
import { writeFileSync, mkdirSync, readdirSync, rmSync, statSync } from "fs";
import { join, dirname, resolve } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DEFAULT_FILE_COUNT = 2000;
const defaultBenchDir = join(__dirname, "__in__");

function parseFileCount(value) {
  return Number.parseInt(value ?? "", 10) || DEFAULT_FILE_COUNT;
}

function createLargeTemplateVariant() {
  const panels = [];
  for (let i = 0; i < 18; i++) {
    const bucket = i % 6;
    panels.push(`    <article class="template-row template-row-${i}" :data-row="${i}" :class="{ selected: activeRow === ${bucket} }">
      <header>
        <p>{{ rows[${bucket}].eyebrow }}</p>
        <h2>{{ rows[${bucket}].title }}</h2>
        <button type="button" @click="activate(${bucket})">Open</button>
      </header>
      <section class="template-row-body">
        <dl>
          <div v-for="stat in rows[${bucket}].stats" :key="'${i}-' + stat.key">
            <dt>{{ stat.label }}</dt>
            <dd>{{ formatValue(stat.value, ${i}) }}</dd>
          </div>
        </dl>
        <ul>
          <li v-for="item in rows[${bucket}].items" :key="'${i}-' + item.id">
            <span>{{ item.name }}</span>
            <strong>{{ item.score + ${i} }}</strong>
            <em v-if="item.score > threshold">above target</em>
            <em v-else>watch</em>
          </li>
        </ul>
      </section>
    </article>`);
  }

  return `<template>
  <main class="large-template-grid">
    <section class="large-template-summary">
      <h1>{{ title }}</h1>
      <p>{{ selectedRow.title }} / {{ selectedRow.items.length }} items</p>
      <button type="button" @click="threshold += 1">Raise Threshold</button>
    </section>
${panels.join("\n")}
  </main>
</template>

<script setup>
import { computed, ref } from 'vue'
const title = ref('Large template __BENCH_ID__')
const activeRow = ref(0)
const threshold = ref(18)
const rows = ref(Array.from({ length: 6 }, (_, rowIndex) => ({
  eyebrow: 'Cluster ' + rowIndex,
  title: 'Template Group ' + rowIndex,
  stats: [
    { key: 'load', label: 'Load', value: rowIndex * 3 + 11 },
    { key: 'queue', label: 'Queue', value: rowIndex * 5 + 7 },
  ],
  items: Array.from({ length: 5 }, (__, itemIndex) => ({
    id: rowIndex + '-' + itemIndex,
    name: 'Item ' + itemIndex,
    score: rowIndex * 10 + itemIndex,
  })),
})))
const selectedRow = computed(() => rows.value[activeRow.value])
function formatValue(value, offset) { return value + offset }
function activate(index) { activeRow.value = index }
</script>

<style scoped>
.large-template-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }
.large-template-summary { grid-column: 1 / -1; padding: 16px; border: 1px solid #d4d4d8; }
.template-row { border: 1px solid #d4d4d8; padding: 12px; }
.template-row.selected { border-color: #2563eb; }
.template-row-body { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
</style>
`;
}

// SFC templates of varying complexity
export const SFC_TEMPLATES = [
  // Simple
  `<template>
  <div>{{ message }}</div>
</template>

<script setup>
import { ref } from 'vue'
const message = ref('Hello World')
</script>
`,
  // With style
  `<template>
  <div class="container">
    <h1>{{ title }}</h1>
    <p>{{ content }}</p>
  </div>
</template>

<script setup>
import { ref } from 'vue'
const title = ref('Title')
const content = ref('Content')
</script>

<style scoped>
.container { padding: 20px; }
h1 { color: #333; }
</style>
`,
  // Complex with v-for and v-if
  `<template>
  <div class="app">
    <header>
      <h1>{{ title }}</h1>
      <nav>
        <a v-for="link in links" :key="link.id" :href="link.url">{{ link.text }}</a>
      </nav>
    </header>
    <main>
      <section v-if="loading">Loading...</section>
      <section v-else>
        <article v-for="item in items" :key="item.id">
          <h2>{{ item.title }}</h2>
          <p>{{ item.body }}</p>
          <button @click="selectItem(item)">Select</button>
        </article>
      </section>
    </main>
    <footer><p>&copy; {{ year }}</p></footer>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
const title = ref('My App')
const loading = ref(false)
const items = ref([])
const links = ref([{ id: 1, url: '/', text: 'Home' }, { id: 2, url: '/about', text: 'About' }])
const year = computed(() => new Date().getFullYear())
function selectItem(item) { console.log('Selected:', item) }
</script>

<style scoped>
.app { max-width: 1200px; margin: 0 auto; }
header { display: flex; justify-content: space-between; }
</style>
`,
  // Dashboard with many bindings
  `<template>
  <div class="dashboard">
    <aside class="sidebar">
      <div class="logo"><img :src="logoUrl" :alt="appName" /><span>{{ appName }}</span></div>
      <nav class="nav-menu">
        <ul>
          <li v-for="item in menuItems" :key="item.id" :class="{ active: item.active }">
            <a :href="item.href" @click.prevent="navigate(item)">
              <span class="icon">{{ item.icon }}</span>
              <span class="label">{{ item.label }}</span>
              <span v-if="item.badge" class="badge">{{ item.badge }}</span>
            </a>
          </li>
        </ul>
      </nav>
    </aside>
    <main class="main-content">
      <section class="stats-grid">
        <div v-for="stat in stats" :key="stat.id" class="stat-card" :style="{ borderColor: stat.color }">
          <div class="stat-icon" :style="{ backgroundColor: stat.color }">{{ stat.icon }}</div>
          <div class="stat-info">
            <span class="stat-value">{{ stat.value }}</span>
            <span class="stat-label">{{ stat.label }}</span>
          </div>
        </div>
      </section>
      <section class="data-table">
        <table>
          <thead><tr><th v-for="col in columns" :key="col.key" @click="sortBy(col.key)">{{ col.label }}</th></tr></thead>
          <tbody><tr v-for="row in paginatedData" :key="row.id"><td v-for="col in columns" :key="col.key">{{ row[col.key] }}</td></tr></tbody>
        </table>
        <div class="pagination">
          <button @click="prevPage" :disabled="currentPage === 1">Prev</button>
          <span>Page {{ currentPage }} of {{ totalPages }}</span>
          <button @click="nextPage" :disabled="currentPage === totalPages">Next</button>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
const appName = ref('Dashboard')
const logoUrl = ref('/logo.png')
const currentPage = ref(1)
const pageSize = ref(10)
const menuItems = ref([
  { id: 1, label: 'Dashboard', icon: 'Chart', href: '/', active: true },
  { id: 2, label: 'Users', icon: 'Users', href: '/users', badge: 5 },
])
const stats = ref([
  { id: 1, label: 'Users', value: '12,345', icon: 'Users', color: '#4CAF50' },
  { id: 2, label: 'Revenue', value: '$54,321', icon: 'Money', color: '#2196F3' },
])
const columns = ref([{ key: 'id', label: 'ID' }, { key: 'name', label: 'Name' }])
const tableData = ref([])
const totalPages = computed(() => Math.ceil(tableData.value.length / pageSize.value))
const paginatedData = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return tableData.value.slice(start, start + pageSize.value)
})
function navigate(item) { menuItems.value.forEach(i => i.active = i.id === item.id) }
function sortBy(key) { console.log('Sort by', key) }
function prevPage() { if (currentPage.value > 1) currentPage.value-- }
function nextPage() { if (currentPage.value < totalPages.value) currentPage.value++ }
onMounted(() => { tableData.value = Array.from({ length: 50 }, (_, i) => ({ id: i + 1, name: 'Item ' + (i + 1) })) })
</script>

<style scoped>
.dashboard { display: flex; min-height: 100vh; }
.sidebar { width: 260px; background: #1a1a2e; color: white; padding: 20px; }
.main-content { flex: 1; padding: 20px; background: #f5f5f5; }
.stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; }
</style>
`,
  // Product page with complex interactions
  `<template>
  <div class="product-page">
    <div class="product-container">
      <div class="product-gallery">
        <div class="main-image">
          <img :src="selectedImage" :alt="product.name" />
          <button @click="prevImage" class="nav-btn prev">Prev</button>
          <button @click="nextImage" class="nav-btn next">Next</button>
        </div>
        <div class="thumbnails">
          <button v-for="(img, index) in product.images" :key="index" @click="selectImage(index)" :class="{ active: selectedImageIndex === index }">
            <img :src="img.thumbnail" :alt="img.alt" />
          </button>
        </div>
      </div>
      <div class="product-info">
        <span class="brand">{{ product.brand }}</span>
        <h1 class="title">{{ product.name }}</h1>
        <div class="rating">
          <span v-for="star in 5" :key="star" class="star" :class="{ filled: star <= product.rating }">Star</span>
          <span class="count">({{ product.reviewCount }} reviews)</span>
        </div>
        <div class="price-section">
          <span v-if="product.originalPrice" class="original-price">\${{ product.originalPrice }}</span>
          <span class="current-price">\${{ product.price }}</span>
        </div>
        <div class="options">
          <div class="color-options">
            <button v-for="color in product.colors" :key="color.name" @click="selectColor(color)" :class="{ selected: selectedColor === color.name }" :style="{ backgroundColor: color.hex }"></button>
          </div>
          <div class="size-options">
            <button v-for="size in product.sizes" :key="size" @click="selectSize(size)" :class="{ selected: selectedSize === size }">{{ size }}</button>
          </div>
          <div class="quantity-selector">
            <button @click="decrementQuantity" :disabled="quantity <= 1">-</button>
            <input v-model.number="quantity" type="number" min="1" :max="product.stock" />
            <button @click="incrementQuantity" :disabled="quantity >= product.stock">+</button>
          </div>
        </div>
        <div class="actions">
          <button @click="addToCart" class="add-to-cart" :disabled="!canAddToCart">Add to Cart - \${{ totalPrice }}</button>
          <button @click="toggleWishlist" class="wishlist" :class="{ active: isWishlisted }">Heart</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
const product = ref({
  name: 'Premium Headphones', brand: 'AudioTech', price: 299.99, originalPrice: 399.99, rating: 4.5, reviewCount: 1234, stock: 15,
  images: [{ full: '/img1.jpg', thumbnail: '/t1.jpg', alt: 'Front' }],
  colors: [{ name: 'Black', hex: '#000' }, { name: 'White', hex: '#FFF' }],
  sizes: ['S', 'M', 'L', 'XL']
})
const selectedImageIndex = ref(0)
const selectedColor = ref('Black')
const selectedSize = ref('M')
const quantity = ref(1)
const isWishlisted = ref(false)
const selectedImage = computed(() => product.value.images[selectedImageIndex.value]?.full)
const totalPrice = computed(() => (product.value.price * quantity.value).toFixed(2))
const canAddToCart = computed(() => selectedColor.value && selectedSize.value && quantity.value > 0)
function selectImage(index) { selectedImageIndex.value = index }
function prevImage() { selectedImageIndex.value = (selectedImageIndex.value - 1 + product.value.images.length) % product.value.images.length }
function nextImage() { selectedImageIndex.value = (selectedImageIndex.value + 1) % product.value.images.length }
function selectColor(color) { selectedColor.value = color.name }
function selectSize(size) { selectedSize.value = size }
function incrementQuantity() { if (quantity.value < product.value.stock) quantity.value++ }
function decrementQuantity() { if (quantity.value > 1) quantity.value-- }
function addToCart() { console.log('Add to cart') }
function toggleWishlist() { isWishlisted.value = !isWishlisted.value }
</script>

<style scoped>
.product-page { max-width: 1400px; margin: 0 auto; padding: 20px; }
.product-container { display: grid; grid-template-columns: 1fr 1fr; gap: 40px; }
.thumbnails { display: flex; gap: 10px; margin-top: 10px; }
.price-section { font-size: 24px; margin: 20px 0; }
.original-price { text-decoration: line-through; color: #999; }
.current-price { color: #e74c3c; font-weight: bold; }
</style>
`,
  // Chat with real-time features
  `<template>
  <div class="chat-container">
    <aside class="conversations-sidebar">
      <div class="search-box"><input v-model="searchQuery" type="text" placeholder="Search..." /></div>
      <ul class="conversation-list">
        <li v-for="conv in filteredConversations" :key="conv.id" :class="{ active: activeConversation?.id === conv.id }" @click="selectConversation(conv)">
          <div class="avatar" :class="{ online: conv.participant.online }"><img :src="conv.participant.avatar" :alt="conv.participant.name" /></div>
          <div class="conv-info">
            <span class="name">{{ conv.participant.name }}</span>
            <span class="time">{{ formatTime(conv.lastMessage.timestamp) }}</span>
            <span v-if="conv.typing" class="typing">typing...</span>
            <span v-else class="last-message">{{ conv.lastMessage.text }}</span>
          </div>
        </li>
      </ul>
    </aside>
    <main class="chat-main">
      <template v-if="activeConversation">
        <header class="chat-header">
          <div class="avatar" :class="{ online: activeConversation.participant.online }"><img :src="activeConversation.participant.avatar" /></div>
          <span class="name">{{ activeConversation.participant.name }}</span>
          <button @click="startCall('voice')">Phone</button>
          <button @click="startCall('video')">Video</button>
        </header>
        <div class="messages-container" ref="messagesContainer">
          <template v-for="(group, date) in groupedMessages" :key="date">
            <div class="date-divider">{{ formatDate(date) }}</div>
            <div v-for="message in group" :key="message.id" class="message" :class="{ sent: message.senderId === currentUser.id }">
              <div class="message-content">
                <div v-if="message.type === 'text'" class="text-message">{{ message.text }}</div>
                <div v-else-if="message.type === 'image'" class="image-message"><img :src="message.imageUrl" @click="openImage(message)" /></div>
                <div class="message-meta">
                  <span class="time">{{ formatTime(message.timestamp) }}</span>
                  <span v-if="message.senderId === currentUser.id" class="status">{{ message.status }}</span>
                </div>
              </div>
              <div class="message-actions">
                <button @click="replyTo(message)">Reply</button>
              </div>
            </div>
          </template>
        </div>
        <footer class="chat-input">
          <button @click="showAttachMenu = !showAttachMenu">Attach</button>
          <textarea v-model="messageText" placeholder="Type a message..." @keydown.enter.exact.prevent="sendMessage"></textarea>
          <button v-if="messageText.trim()" @click="sendMessage">Send</button>
        </footer>
      </template>
    </main>
  </div>
</template>

<script setup>
import { ref, computed, nextTick } from 'vue'
const currentUser = ref({ id: 1, name: 'Me', avatar: '/me.jpg' })
const conversations = ref([
  { id: 1, participant: { id: 2, name: 'Alice', avatar: '/alice.jpg', online: true }, lastMessage: { text: 'Hey!', timestamp: new Date() }, typing: false },
  { id: 2, participant: { id: 3, name: 'Bob', avatar: '/bob.jpg', online: false }, lastMessage: { text: 'See you!', timestamp: new Date(Date.now() - 7200000) }, typing: false },
])
const messages = ref([
  { id: 1, conversationId: 1, senderId: 2, type: 'text', text: 'Hey!', timestamp: new Date(Date.now() - 86400000), status: 'read' },
  { id: 2, conversationId: 1, senderId: 1, type: 'text', text: 'Hi Alice!', timestamp: new Date(Date.now() - 86300000), status: 'read' },
])
const searchQuery = ref('')
const activeConversation = ref(null)
const messageText = ref('')
const showAttachMenu = ref(false)
const messagesContainer = ref(null)
const filteredConversations = computed(() => searchQuery.value ? conversations.value.filter(c => c.participant.name.toLowerCase().includes(searchQuery.value.toLowerCase())) : conversations.value)
const currentMessages = computed(() => activeConversation.value ? messages.value.filter(m => m.conversationId === activeConversation.value.id) : [])
const groupedMessages = computed(() => { const g = {}; currentMessages.value.forEach(m => { const d = new Date(m.timestamp).toDateString(); if (!g[d]) g[d] = []; g[d].push(m) }); return g })
function selectConversation(conv) { activeConversation.value = conv; nextTick(() => scrollToBottom()) }
function sendMessage() { if (!messageText.value.trim()) return; messages.value.push({ id: Date.now(), conversationId: activeConversation.value.id, senderId: currentUser.value.id, type: 'text', text: messageText.value, timestamp: new Date(), status: 'sent' }); messageText.value = ''; nextTick(() => scrollToBottom()) }
function scrollToBottom() { if (messagesContainer.value) messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight }
function formatTime(date) { return new Date(date).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }
function formatDate(date) { return new Date(date).toLocaleDateString() }
function startCall(type) { console.log('Start', type, 'call') }
function replyTo(message) { console.log('Reply to', message.id) }
function openImage(message) { console.log('Open image', message.imageUrl) }
</script>

<style scoped>
.chat-container { display: grid; grid-template-columns: 320px 1fr; height: 100vh; }
.conversations-sidebar { background: #f8f9fa; border-right: 1px solid #e0e0e0; }
.conversation-list li { display: flex; padding: 15px; cursor: pointer; border-bottom: 1px solid #eee; }
.conversation-list li.active { background: #e3f2fd; }
.chat-main { display: flex; flex-direction: column; }
.messages-container { flex: 1; overflow-y: auto; padding: 20px; }
.message { display: flex; margin-bottom: 15px; max-width: 70%; }
.message.sent { margin-left: auto; }
.message-content { background: #f1f1f1; padding: 10px 15px; border-radius: 18px; }
.message.sent .message-content { background: #0084ff; color: white; }
.chat-input { display: flex; align-items: center; padding: 15px; border-top: 1px solid #e0e0e0; gap: 10px; }
.chat-input textarea { flex: 1; padding: 10px; border: 1px solid #ddd; border-radius: 20px; resize: none; }
</style>
`,
  // Options API with props, data, computed, watchers, and methods
  `<template>
  <section class="options-workspace">
    <header>
      <p>{{ subtitle }}</p>
      <h1>{{ title }}</h1>
      <button type="button" @click="toggleArchived">{{ showArchived ? 'Hide' : 'Show' }} Archived</button>
    </header>
    <form class="filters" @submit.prevent="applyFilter">
      <input v-model.trim="draftQuery" :placeholder="placeholder" />
      <select v-model="selectedStatus">
        <option v-for="status in statuses" :key="status" :value="status">{{ status }}</option>
      </select>
      <button type="submit">Apply</button>
    </form>
    <ul class="ticket-list">
      <li v-for="ticket in visibleTickets" :key="ticket.id" :class="{ overdue: ticket.due < today }">
        <span>{{ ticket.code }}</span>
        <strong>{{ ticket.summary }}</strong>
        <small>{{ ticket.owner }} / {{ ticket.status }}</small>
        <button type="button" @click="assign(ticket.id, fallbackOwner)">Assign</button>
      </li>
    </ul>
    <footer>{{ visibleTickets.length }} of {{ tickets.length }} tickets</footer>
  </section>
</template>

<script>
export default {
  name: 'OptionsBenchmark__BENCH_ID__',
  props: {
    initialStatus: { type: String, default: 'open' },
    fallbackOwner: { type: String, default: 'Platform' },
  },
  data() {
    return {
      title: 'Options API board __BENCH_ID__',
      subtitle: 'Synthetic workload',
      placeholder: 'Filter tickets',
      draftQuery: '',
      query: '',
      selectedStatus: this.initialStatus,
      showArchived: false,
      today: 20,
      statuses: ['open', 'blocked', 'review', 'done'],
      tickets: [
        { id: 1, code: 'OPS-1', summary: 'Review rollout', owner: 'Aki', status: 'open', archived: false, due: 12 },
        { id: 2, code: 'OPS-2', summary: 'Trim queue', owner: 'Mika', status: 'blocked', archived: false, due: 25 },
        { id: 3, code: 'OPS-3', summary: 'Close incident', owner: 'Ren', status: 'done', archived: true, due: 10 },
      ],
    }
  },
  computed: {
    normalizedQuery() {
      return this.query.toLowerCase()
    },
    visibleTickets() {
      return this.tickets.filter((ticket) => {
        const matchesStatus = ticket.status === this.selectedStatus || this.selectedStatus === 'open'
        const matchesQuery = !this.normalizedQuery || ticket.summary.toLowerCase().includes(this.normalizedQuery)
        const matchesArchive = this.showArchived || !ticket.archived
        return matchesStatus && matchesQuery && matchesArchive
      })
    },
  },
  watch: {
    selectedStatus() {
      this.query = ''
      this.draftQuery = ''
    },
  },
  methods: {
    applyFilter() {
      this.query = this.draftQuery
    },
    toggleArchived() {
      this.showArchived = !this.showArchived
    },
    assign(id, owner) {
      const ticket = this.tickets.find((item) => item.id === id)
      if (ticket) ticket.owner = owner
    },
  },
}
</script>

<style scoped>
.options-workspace { display: grid; gap: 16px; padding: 20px; }
.filters { display: flex; gap: 8px; }
.ticket-list { display: grid; gap: 8px; padding: 0; list-style: none; }
.ticket-list li { display: grid; grid-template-columns: 80px 1fr auto auto; gap: 12px; align-items: center; }
.ticket-list li.overdue { color: #b91c1c; }
</style>
`,
  // TypeScript-heavy script setup with generics, unions, and typed emits
  `<template>
  <section class="typed-resource">
    <header>
      <h1>{{ headline }}</h1>
      <p>{{ benchmarkToken }}</p>
      <button type="button" @click="cycleState">Cycle</button>
    </header>
    <article v-for="resource in decoratedResources" :key="resource.id" :data-kind="resource.state.kind">
      <h2>{{ resource.name }}</h2>
      <p>{{ resource.summary }}</p>
      <meter :value="resource.score" min="0" max="100"></meter>
      <button type="button" @click="select(resource)">Select</button>
      <template v-if="resource.state.kind === 'failed'">
        <strong>{{ resource.state.reason }}</strong>
      </template>
      <template v-else-if="resource.state.kind === 'ready'">
        <span>{{ resource.state.deployedAt }}</span>
      </template>
      <template v-else>
        <span>{{ resource.state.percent }}%</span>
      </template>
    </article>
    <footer>{{ selected?.name ?? 'None selected' }} / {{ totals.ready }} ready</footer>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

type ResourceState =
  | { kind: 'ready'; deployedAt: string }
  | { kind: 'pending'; percent: number }
  | { kind: 'failed'; reason: string }

type Resource<TMeta extends Record<string, unknown>> = {
  id: number
  name: string
  weight: number
  state: ResourceState
  meta: TMeta
}

type DecoratedResource<TMeta extends Record<string, unknown>> = Resource<TMeta> & {
  score: number
  summary: string
}

const emit = defineEmits<{
  selected: [payload: DecoratedResource<{ owner: string; tags: string[] }>]
}>()

const benchmarkToken = 'ts-heavy-__BENCH_ID__' as const
const headline = computed(() => 'Typed resources ' + benchmarkToken)
const selected = ref<DecoratedResource<{ owner: string; tags: string[] }> | null>(null)
const resources = ref<Resource<{ owner: string; tags: string[] }>[]>([
  { id: 1, name: 'Compiler', weight: 0.85, state: { kind: 'ready', deployedAt: '2026-01-01' }, meta: { owner: 'Core', tags: ['sfc', 'fast'] } },
  { id: 2, name: 'Checker', weight: 0.65, state: { kind: 'pending', percent: 42 }, meta: { owner: 'Types', tags: ['ts', 'diagnostics'] } },
  { id: 3, name: 'Linter', weight: 0.45, state: { kind: 'failed', reason: 'rule mismatch' }, meta: { owner: 'Lint', tags: ['rules'] } },
])

function scoreFor<TMeta extends Record<string, unknown>>(resource: Resource<TMeta>): number {
  const stateScore: Record<ResourceState['kind'], number> = { ready: 100, pending: 60, failed: 15 }
  return Math.round(stateScore[resource.state.kind] * resource.weight)
}

const decoratedResources = computed(() =>
  resources.value.map((resource): DecoratedResource<{ owner: string; tags: string[] }> => ({
    ...resource,
    score: scoreFor(resource),
    summary: resource.meta.owner + ' / ' + resource.meta.tags.join(', '),
  })),
)

const totals = computed(() => decoratedResources.value.reduce(
  (acc, resource) => {
    acc[resource.state.kind] += 1
    return acc
  },
  { ready: 0, pending: 0, failed: 0 } as Record<ResourceState['kind'], number>,
))

function select(resource: DecoratedResource<{ owner: string; tags: string[] }>): void {
  selected.value = resource
  emit('selected', resource)
}

function cycleState(): void {
  resources.value = resources.value.map((resource) => ({
    ...resource,
    state: resource.state.kind === 'ready'
      ? { kind: 'pending', percent: 20 }
      : resource.state.kind === 'pending'
        ? { kind: 'failed', reason: 'synthetic transition' }
        : { kind: 'ready', deployedAt: '2026-01-02' },
  }))
}
</script>

<style scoped>
.typed-resource { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; }
.typed-resource header, .typed-resource footer { grid-column: 1 / -1; }
.typed-resource article { border: 1px solid #d1d5db; padding: 12px; }
</style>
`,
  // Large template inside the many-file corpus, distinct from compare-tools' single large-SFC lane
  createLargeTemplateVariant(),
  // Class-style SFC with a default exported TypeScript class and template bindings
  `<template>
  <section class="class-component">
    <header>
      <h1>{{ view.label }}</h1>
      <p>{{ summary }}</p>
      <button type="button" @click="increment">Increment</button>
    </header>
    <ol>
      <li v-for="step in steps" :key="step.id" :class="{ done: step.done }">
        <span>{{ step.title }}</span>
        <button type="button" @click="toggle(step.id)">{{ step.done ? 'Undo' : 'Done' }}</button>
      </li>
    </ol>
    <footer>{{ completedCount }} / {{ steps.length }} complete</footer>
  </section>
</template>

<script lang="ts">
class StepModel {
  constructor(
    public id: number,
    public title: string,
    public done: boolean,
  ) {}
}

class ClassViewModel {
  label = 'Class component __BENCH_ID__'
  count = 1
}

export default class ClassBenchComponent__BENCH_ID__ {
  view = new ClassViewModel()
  steps = [
    new StepModel(1, 'Parse script block', true),
    new StepModel(2, 'Compile template', false),
    new StepModel(3, 'Attach render function', false),
  ]

  get completedCount(): number {
    return this.steps.filter((step) => step.done).length
  }

  get summary(): string {
    return this.view.label + ' #' + this.view.count
  }

  increment(): void {
    this.view.count += 1
  }

  toggle(id: number): void {
    const step = this.steps.find((item) => item.id === id)
    if (step) step.done = !step.done
  }
}
</script>

<style scoped>
.class-component { display: grid; gap: 12px; padding: 20px; }
.class-component li { display: flex; justify-content: space-between; gap: 12px; }
.class-component li.done { text-decoration: line-through; }
</style>
`,
];

// Every generated body must be unique. Repeating identical bodies lets
// content-addressed compile caches (e.g. the `vize build --format stats`
// dedup cache) serve most of the corpus from a hash lookup, so the benchmark
// measures cache hits instead of compilation. A per-file token inside an
// existing string literal keeps each complexity tier intact while forcing a
// real compile per file.
export function uniquify(template, index) {
  const id = String(index).padStart(4, "0");
  if (template.includes("__BENCH_ID__")) {
    return template.replaceAll("__BENCH_ID__", id);
  }

  const marked = template.replace(/ref\('([^']*)'\)/, (_, text) => `ref('${text} ${id}')`);
  if (marked === template) {
    throw new Error("template has no __BENCH_ID__ marker or ref('...') anchor to uniquify");
  }
  return marked;
}

export function generateCorpus({
  fileCount = DEFAULT_FILE_COUNT,
  benchDir = defaultBenchDir,
  log = console.log,
} = {}) {
  const normalizedFileCount = Math.max(0, Math.trunc(Number(fileCount) || 0));
  const writeLog = typeof log === "function" ? log : () => {};

  // Ensure directory exists
  mkdirSync(benchDir, { recursive: true });

  writeLog(`Generating ${normalizedFileCount} SFC files in ${benchDir}...`);

  for (const file of readdirSync(benchDir)) {
    if (file.startsWith("Component") && file.endsWith(".vue")) {
      rmSync(join(benchDir, file), { force: true });
    }
  }

  for (let i = 0; i < normalizedFileCount; i++) {
    const template = uniquify(SFC_TEMPLATES[i % SFC_TEMPLATES.length], i);
    const filename = `Component${String(i).padStart(4, "0")}.vue`;
    const filepath = join(benchDir, filename);
    writeFileSync(filepath, template);

    if ((i + 1) % 500 === 0) {
      writeLog(`  Generated ${i + 1} files...`);
    }
  }

  writeLog(`Done! Generated ${normalizedFileCount} SFC files.`);

  // Calculate total size
  const files = readdirSync(benchDir).filter((f) => f.endsWith(".vue"));
  const totalSize = files.reduce((sum, f) => sum + statSync(join(benchDir, f)).size, 0);
  writeLog(`Total size: ${(totalSize / 1024 / 1024).toFixed(2)} MB`);

  // Generate tsconfig.json for vue-tsc / vize check
  const tsconfig = {
    compilerOptions: {
      target: "ESNext",
      module: "ESNext",
      moduleResolution: "bundler",
      strict: true,
      jsx: "preserve",
      noEmit: true,
      skipLibCheck: true,
      paths: {
        vue: ["../node_modules/vue"],
      },
    },
    include: ["./*.vue"],
  };
  writeFileSync(join(benchDir, "tsconfig.json"), JSON.stringify(tsconfig, null, 2));
  writeLog("Generated tsconfig.json");

  // Generate eslint.config.mjs for eslint-plugin-vue
  const eslintConfig = `import pluginVue from "eslint-plugin-vue";

export default [
  ...pluginVue.configs["flat/recommended"],
  {
    files: ["*.vue"],
    rules: {
      "vue/multi-word-component-names": "off",
    },
  },
];
`;
  writeFileSync(join(benchDir, "eslint.config.mjs"), eslintConfig);
  writeLog("Generated eslint.config.mjs");

  // Generate vize.config.json so benchmark runs cover shared config loading.
  const vizeConfig = {
    linter: {
      rules: {
        "vue/prop-name-casing": "off",
      },
    },
    typeChecker: {
      checkProps: true,
      checkTemplateBindings: true,
      checkEmits: true,
    },
  };
  writeFileSync(join(benchDir, "vize.config.json"), `${JSON.stringify(vizeConfig, null, 2)}\n`);
  writeLog("Generated vize.config.json");

  // Generate vite entry file for vite-plugin benchmark
  const viteEntryImports = [];
  const viteEntryComponents = [];
  const entryCount = normalizedFileCount; // import all files for fair vite benchmark
  for (let i = 0; i < entryCount; i++) {
    const name = `Component${String(i).padStart(4, "0")}`;
    viteEntryImports.push(`import ${name} from './${name}.vue'`);
    viteEntryComponents.push(name);
  }
  const viteEntry = `${viteEntryImports.join("\n")}
import { createApp, h } from 'vue'

const app = createApp({
  render() {
    return h('div', [${viteEntryComponents.map((c) => `h(${c})`).join(", ")}])
  }
})
app.mount('#app')
`;
  writeFileSync(join(benchDir, "main.ts"), viteEntry);
  writeLog(`Generated main.ts (imports ${entryCount} components)`);

  // Generate index.html for vite
  const indexHtml = `<!DOCTYPE html>
<html>
<head><title>Bench</title></head>
<body>
  <div id="app"></div>
  <script type="module" src="./main.ts"></script>
</body>
</html>
`;
  writeFileSync(join(benchDir, "index.html"), indexHtml);
  writeLog("Generated index.html");

  return {
    dir: benchDir,
    fileCount: normalizedFileCount,
    totalSize,
  };
}

export function main(argv = process.argv.slice(2)) {
  return generateCorpus({ fileCount: parseFileCount(argv[0]) });
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  main();
}
