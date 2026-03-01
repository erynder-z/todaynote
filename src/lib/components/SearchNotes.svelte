<script lang="ts">
  import { t } from '$lib';
</script>

<div class="search-placeholder">
  <input type="text" placeholder={$t('search.title')} class="theme-input" />
  <p class="muted">Search functionality coming soon!</p>
</div>

<style>
  .search-placeholder {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    align-items: center;
    padding: 1rem;
  }

  .theme-input {
    width: 100%;
    padding: 0.8rem 1.2rem;
    border-radius: 0.625rem;
    border: 0.0625rem solid var(--border);
    background-color: var(--bg-surface);
    color: var(--text-main);
    font-size: 1rem;
  }

  .muted {
    color: var(--text-muted);
  }
</style>
