/**
 * HTML structure and inline JS generation for the Musea gallery.
 *
 * Extracted from gallery.ts to keep file sizes manageable.
 */

import { serializeScriptValue } from "../security.js";

/**
 * Generate the gallery HTML body (header, sidebar, content, and inline script).
 */
export function generateGalleryBody(basePath: string): string {
  return `
  <header class="header">
    <div class="header-left">
      <a href="${basePath}" class="logo">
        <svg class="logo-svg" width="32" height="32" viewBox="0 0 200 200" fill="none">
          <g transform="translate(30, 25) scale(1.2)">
            <g transform="translate(15, 10) skewX(-15)">
              <path d="M 65 0 L 40 60 L 70 20 L 65 0 Z" fill="currentColor"/>
              <path d="M 20 0 L 40 60 L 53 13 L 20 0 Z" fill="currentColor"/>
            </g>
          </g>
          <g transform="translate(110, 120)">
            <line x1="5" y1="10" x2="5" y2="50" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
            <line x1="60" y1="10" x2="60" y2="50" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
            <path d="M 0 10 L 32.5 0 L 65 10" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
            <rect x="15" y="18" width="14" height="12" rx="1" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.7"/>
            <rect x="36" y="18" width="14" height="12" rx="1" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.7"/>
            <rect x="23" y="35" width="18" height="12" rx="1" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.6"/>
          </g>
        </svg>
        Musea
      </a>
      <span class="header-subtitle">Component Gallery</span>
    </div>
    <div class="search-container">
      <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
      </svg>
      <input type="text" class="search-input" placeholder="Search components..." id="search">
    </div>
  </header>

  <main class="main">
    <aside class="sidebar" id="sidebar">
      <div class="loading">
        <div class="loading-spinner"></div>
        Loading...
      </div>
    </aside>
    <section class="content" id="content">
      <div class="empty-state">
        <div class="empty-state-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M4 5a1 1 0 0 1 1-1h14a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5Z"/>
            <path d="M4 13a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-6Z"/>
            <path d="M16 13a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-2a1 1 0 0 1-1-1v-6Z"/>
          </svg>
        </div>
        <div class="empty-state-title">Select a component</div>
        <div class="empty-state-text">Choose a component from the sidebar to view its variants and documentation</div>
      </div>
    </section>
  </main>`;
}

/**
 * Generate the gallery inline script (SPA logic).
 */
export function generateGalleryScript(basePath: string): string {
  return `
    const basePath = ${serializeScriptValue(basePath)};
    let arts = [];
    let selectedArt = null;
    let searchQuery = '';

    async function loadArts() {
      try {
        const res = await fetch(basePath + '/api/arts');
        arts = await res.json();
        renderSidebar();
      } catch (e) {
        console.error('Failed to load arts:', e);
        document.getElementById('sidebar').innerHTML = '<div class="loading">Failed to load</div>';
      }
    }

    function renderSidebar() {
      const sidebar = document.getElementById('sidebar');
      const categories = {};

      const filtered = searchQuery
        ? arts.filter(a => a.metadata.title.toLowerCase().includes(searchQuery.toLowerCase()))
        : arts;

      for (const art of filtered) {
        const cat = art.metadata.category || 'Components';
        if (!categories[cat]) categories[cat] = [];
        categories[cat].push(art);
      }

      if (Object.keys(categories).length === 0) {
        sidebar.innerHTML = '<div class="sidebar-section"><div class="loading">No components found</div></div>';
        return;
      }

      let html = '';
      for (const [category, items] of Object.entries(categories)) {
        const escapedCategory = escapeHtml(category);
        html += '<div class="sidebar-section">';
        html += '<div class="category-header" data-category="' + escapedCategory + '">';
        html += '<svg class="category-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg>';
        html += '<span>' + escapedCategory + '</span>';
        html += '<span class="category-count">' + items.length + '</span>';
        html += '</div>';
        html += '<ul class="art-list" data-category="' + escapedCategory + '">';
        for (const art of items) {
          const active = selectedArt?.path === art.path ? 'active' : '';
          const variantCount = art.variants?.length || 0;
          html += '<li class="art-item ' + active + '" data-path="' + escapeHtml(art.path) + '">';
          html += '<span>' + escapeHtml(art.metadata.title) + '</span>';
          html += '<span class="art-variant-count">' + variantCount + ' variant' + (variantCount !== 1 ? 's' : '') + '</span>';
          html += '</li>';
        }
        html += '</ul>';
        html += '</div>';
      }

      sidebar.innerHTML = html;

      sidebar.querySelectorAll('.art-item').forEach(item => {
        item.addEventListener('click', () => {
          const artPath = item.dataset.path;
          selectedArt = arts.find(a => a.path === artPath);
          renderSidebar();
          renderContent();
        });
      });

      sidebar.querySelectorAll('.category-header').forEach(header => {
        header.addEventListener('click', () => {
          header.classList.toggle('collapsed');
          const list = header.parentElement?.querySelector('.art-list');
          if (list) list.style.display = header.classList.contains('collapsed') ? 'none' : 'block';
        });
      });
    }

    function renderContent() {
      const content = document.getElementById('content');
      if (!selectedArt) {
        content.innerHTML = \`
          <div class="empty-state">
            <div class="empty-state-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M4 5a1 1 0 0 1 1-1h14a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5Z"/>
                <path d="M4 13a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-6Z"/>
                <path d="M16 13a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-2a1 1 0 0 1-1-1v-6Z"/>
              </svg>
            </div>
            <div class="empty-state-title">Select a component</div>
            <div class="empty-state-text">Choose a component from the sidebar to view its variants</div>
          </div>
        \`;
        return;
      }

      const meta = selectedArt.metadata;
      const tags = meta.tags || [];
      const variantCount = selectedArt.variants?.length || 0;

      let html = '<div class="content-inner">';
      html += '<div class="content-header">';
      html += '<h1 class="content-title">' + escapeHtml(meta.title) + '</h1>';
      if (meta.description) {
        html += '<p class="content-description">' + escapeHtml(meta.description) + '</p>';
      }
      html += '<div class="content-meta">';
      html += '<span class="meta-tag"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>' + variantCount + ' variant' + (variantCount !== 1 ? 's' : '') + '</span>';
      if (meta.category) {
        html += '<span class="meta-tag"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>' + escapeHtml(meta.category) + '</span>';
      }
      for (const tag of tags) {
        html += '<span class="meta-tag">#' + escapeHtml(tag) + '</span>';
      }
      html += '</div>';
      html += '</div>';

      html += '<div class="gallery">';
      for (const variant of selectedArt.variants) {
        const previewUrl = basePath + '/preview?art=' + encodeURIComponent(selectedArt.path) + '&variant=' + encodeURIComponent(variant.name);

        html += '<div class="variant-card">';
        html += '<div class="variant-preview">';
        html += '<iframe src="' + previewUrl + '" loading="lazy" title="' + escapeHtml(variant.name) + '"></iframe>';
        html += '</div>';
        html += '<div class="variant-info">';
        html += '<div>';
        html += '<span class="variant-name">' + escapeHtml(variant.name) + '</span>';
        if (variant.isDefault) html += ' <span class="variant-badge">Default</span>';
        html += '</div>';
        html += '<div class="variant-actions">';
        html += '<button class="variant-action-btn" title="Open in new tab" onclick="window.open(\\'' + previewUrl + '\\', \\'_blank\\')"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg></button>';
        html += '</div>';
        html += '</div>';
        html += '</div>';
      }
      html += '</div>';
      html += '</div>';

      content.innerHTML = html;
    }

    function escapeHtml(str) {
      if (!str) return '';
      return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }

    // Search
    document.getElementById('search').addEventListener('input', (e) => {
      searchQuery = e.target.value;
      renderSidebar();
    });

    // Keyboard shortcut for search
    document.addEventListener('keydown', (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        document.getElementById('search').focus();
      }
    });

    loadArts();`;
}
