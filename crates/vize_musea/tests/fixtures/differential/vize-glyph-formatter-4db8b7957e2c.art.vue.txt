
<div class="app-container" id="main-app" data-testid="app">
  <header class="header" :class="{ 'header--sticky': isSticky }">
    <nav class="nav">
      <ul class="nav-list">
        <li v-for="item in navItems" :key="item.id" class="nav-item">
          <router-link :to="item.path" class="nav-link">{{ item.label }}</router-link>
        </li>
      </ul>
    </nav>
  </header>
  <main class="main-content">
    <section v-if="loading" class="loading-section">
      <div class="spinner"></div>
      <p>Loading...</p>
    </section>
    <section v-else-if="error" class="error-section">
      <p class="error-message">{{ error }}</p>
      <button @click="retry" class="btn btn-primary">Retry</button>
    </section>
    <section v-else class="content-section">
      <article v-for="post in posts" :key="post.id" class="post-card">
        <h2 class="post-title">{{ post.title }}</h2>
        <p class="post-excerpt">{{ post.excerpt }}</p>
        <footer class="post-footer">
          <span class="post-date">{{ formatDate(post.date) }}</span>
          <router-link :to="`/posts/${post.id}`" class="read-more">Read more</router-link>
        </footer>
      </article>
    </section>
  </main>
  <footer class="footer">
    <p>&copy; 2024 My App</p>
  </footer>
</div>
