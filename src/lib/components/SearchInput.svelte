<script lang="ts">
  /**
   * Search input component with integrated actions (fuzzy toggle, clear).
   */
  import { t } from '$lib';
  import type { SearchInputProps } from '$lib/interfaces/ui';

  let {
    query = $bindable(),
    isFuzzy = $bindable(),
    searchMode,
    selectedTag,
    onInput,
    onClearTag,
    onClearQuery,
    onToggleFuzzy,
  }: SearchInputProps = $props();
</script>

<header class="main-header">
  <div class="search-input-container">
    <div class="search-icon">
      <svg
        viewBox="0 0 24 24"
        width="20"
        height="20"
        stroke="currentColor"
        stroke-width="2"
        fill="none"
      >
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
    </div>

    {#if selectedTag}
      <button class="tag-breadcrumb" onclick={onClearTag}>
        #{selectedTag}
        <span class="clear">×</span>
      </button>
    {/if}

    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="text"
      bind:value={query}
      oninput={onInput}
      placeholder={searchMode === 'notes' || selectedTag
        ? $t('search.start_typing')
        : searchMode === 'threads'
          ? $t('search.start_typing_threads')
          : $t('search.start_typing_tags')}
      spellcheck="false"
      autofocus
    />

    <div class="input-actions">
      <button
        class="fuzzy-toggle"
        class:active={isFuzzy}
        onclick={() => {
          if (onToggleFuzzy) onToggleFuzzy();
          else isFuzzy = !isFuzzy;
        }}
        title={isFuzzy ? $t('search.fuzzy_on') : $t('search.fuzzy_off')}
      >
        {$t('search.fuzzy')}
      </button>

      {#if query}
        <button class="clear-btn" onclick={onClearQuery}> × </button>
      {/if}
    </div>
  </div>
</header>

<style>
  .main-header {
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .search-input-container {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    padding: 0 1rem;
    height: 3rem;
    transition: all 0.2s cubic-bezier(0.2, 0, 0, 1);
  }

  .search-input-container:focus-within {
    border-color: var(--accent);
    background-color: var(--bg-main);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .search-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .search-input-container input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-main);
    font-size: 1rem;
    font-weight: 500;
    outline: none;
    width: 100%;
    padding: 0.75rem 0;
  }

  .input-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .fuzzy-toggle {
    background: none;
    border: 1px solid var(--border);
    border-radius: 0.35rem;
    color: var(--text-muted);
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.15rem 0.4rem;
    cursor: pointer;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
    user-select: none;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .fuzzy-toggle:hover {
    color: var(--text-main);
    border-color: var(--text-muted);
  }

  .fuzzy-toggle.active {
    background-color: var(--accent);
    color: var(--accent-text);
    border-color: var(--accent);
  }

  .clear-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 1.25rem;
    cursor: pointer;
    padding: 0.2rem;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 50%;
    transition: background-color 0.15s;
  }

  .clear-btn:hover {
    color: var(--text-main);
    background-color: var(--bg-surface);
  }

  .tag-breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background-color: var(--accent);
    color: var(--accent-text);
    padding: 0.2rem 0.6rem;
    border-radius: 0.4rem;
    font-size: 0.85rem;
    font-weight: 700;
    border: none;
    cursor: pointer;
    white-space: nowrap;
  }
</style>
