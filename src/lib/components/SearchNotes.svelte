<script lang="ts">
  import { t } from '$lib';
</script>

<div class="search-placeholder">
  <input type="text" placeholder={$t('search.title')} class="search-input" />
  <p>Search functionality coming soon!</p>
</div>

<style>
  .search-placeholder {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: center;
    padding: 1rem;
  }

  .search-input {
    width: 100%;
    padding: 0.8rem;
    border-radius: 8px;
    border: 1px solid #ccc;
    font-size: 1rem;
  }

  @media (prefers-color-scheme: dark) {
    .search-input {
      background-color: #3a3a3a;
      border-color: #555;
      color: #f6f6f6;
    }
  }
</style>
