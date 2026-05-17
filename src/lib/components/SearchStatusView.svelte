<script lang="ts">
  /**
   * Status view component showing loading, empty, and no-results states.
   */
  import { settings, t } from '$lib';
  import type { SearchStatusViewProps } from '$lib/interfaces/ui';

  let { isSearching, hasResults, query, selectedTag }: SearchStatusViewProps =
    $props();

  let showLoading = $derived(isSearching && !hasResults);
  let showNoResults = $derived(
    !isSearching && (query.trim().length > 0 || selectedTag) && !hasResults,
  );
  let showEmpty = $derived(
    !isSearching && query.trim().length === 0 && !selectedTag && !hasResults,
  );
</script>

{#if showLoading}
  <div class="status-view">
    <div class="spinner"></div>
    <p>{$t('search.searching')}</p>
  </div>
{:else if showNoResults}
  <div class="status-view">
    <p class="muted">{$t('search.no_results')}</p>
  </div>
{:else if showEmpty}
  <div class="status-view empty">
    <div class="empty-icon">
      {#if settings.notesListLayout === 'masonry'}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="6rem"
          viewBox="0 -960 960 960"
          width="6rem"
          fill="currentColor"
        >
          <path
            d="M120-520v-320h320v320H120Zm0 400v-320h320v320H120Zm400-400v-320h320v320H520Zm0 400v-320h320v320H520ZM200-600h160v-160H200v160Zm400 0h160v-160H600v160Zm0 400h160v-160H600v160Zm-400 0h160v-160H200v160Zm400-400Zm0 240Zm-240 0Zm0-240Z"
          />
        </svg>
      {:else}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="6rem"
          viewBox="0 -960 960 960"
          width="6rem"
          fill="currentColor"
        >
          <path
            d="M120-240v-80h480v80H120Zm0-200v-80h720v80H120Zm0-200v-80h720v80H120Z"
          />
        </svg>
      {/if}
    </div>
  </div>
{/if}

<style>
  .status-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1.5rem;
    color: var(--text-muted);
  }

  .status-view.empty {
    opacity: 0.5;
  }

  .empty-icon {
    color: var(--border);
    margin-bottom: 0.5rem;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .muted {
    font-style: italic;
    font-size: 1.1rem;
  }
</style>
