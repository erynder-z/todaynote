<script lang="ts">
  /**
   * Enhanced note search component with command-palette aesthetics
   * and full keyboard navigation.
   */
  import {
    ListNavigator,
    ModalFooter,
    NoteSearchResults,
    sessionState,
    settings,
    ThreadSearchResults,
    t,
    toast,
  } from '$lib';
  import { inputManager } from '$lib/stores/input.svelte';
  import type { SearchResult, ThreadSearchResult } from '$lib/types/notes';
  import {
    aggregateThread,
    readNoteContent,
    searchNotes,
    searchThreads,
  } from '$lib/utils/notes';

  let query = $state('');
  let isFuzzy = $state(true);
  let searchMode = $state<'notes' | 'threads'>('notes');
  let results = $state<SearchResult[] | ThreadSearchResult[]>([]);
  let isSearching = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const nav = new ListNavigator(
    () => results.length,
    (i) => {
      if (searchMode === 'notes') {
        selectResult(results[i] as SearchResult);
      } else {
        selectThread(results[i] as ThreadSearchResult);
      }
    },
  );

  $effect(() => {
    return inputManager.registerActions({
      toggleFuzzy: () => {
        isFuzzy = !isFuzzy;
      },
      toggleSearchMode: () => {
        searchMode = searchMode === 'notes' ? 'threads' : 'notes';
        performSearch();
      },
    });
  });

  /**
   * Performs the search operation based on the current query and mode.
   * Debounced via onInput to prevent excessive API calls.
   */
  const performSearch = async () => {
    if (query.trim().length === 0 && searchMode === 'notes') {
      results = [];
      nav.reset();
      return;
    }

    isSearching = true;
    if (searchMode === 'notes') {
      results = await searchNotes(query, isFuzzy);
    } else {
      results = await searchThreads(query, isFuzzy);
    }
    isSearching = false;
    nav.reset();
  };

  /**
   * Handles input changes with debouncing.
   */
  const onInput = () => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(performSearch, 200);
  };

  /**
   * Loads and opens a specific note from a search result.
   */
  const selectResult = async (result: SearchResult) => {
    if (!settings.notesFolder || !result) return;
    const path = `${settings.notesFolder}/${result.filename}`;
    const content = await readNoteContent(path);
    if (content !== null) {
      sessionState.todayNotePath = path;
      sessionState.todayNoteContent = content;
      sessionState.activePopup = null;
    } else {
      toast.error($t('notes.error.load'));
    }
  };

  /**
   * Aggregates and opens a thread view from a search result.
   */
  const selectThread = async (thread: ThreadSearchResult) => {
    if (!thread) return;
    const aggregation = await aggregateThread(thread.name);
    if (aggregation) {
      sessionState.aggregatedThread = aggregation;
      sessionState.activePopup = 'threadAggregation';
    } else {
      toast.error('Failed to aggregate thread');
    }
  };

  /**
   * Handles global keydown events for list navigation.
   */
  const handleKeydown = (e: KeyboardEvent) => {
    if (results.length > 0) {
      nav.handleKey(e);
    }
  };

  $effect(() => {
    // Re-run search if fuzzy mode changes
    if (isFuzzy !== undefined) performSearch();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="search-container" onkeydown={handleKeydown}>
  <header class="search-header">
    <div class="toolbar">
      <div class="mode-tabs">
        <button
          class="tab"
          class:active={searchMode === 'notes'}
          onclick={() => {
            searchMode = 'notes';
            performSearch();
          }}
        >
          {$t('search.notes')}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="1rem"
            viewBox="0 -960 960 960"
            width="1rem"
            fill="currentColor"
            ><path
              d="M280-160v-441q0-33 24-56t57-23h439q33 0 56.5 23.5T880-600v320L680-80H360q-33 0-56.5-23.5T280-160ZM81-710q-6-33 13-59.5t52-32.5l434-77q33-6 59.5 13t32.5 52l10 54h-82l-7-40-433 77 40 226v279q-16-9-27.5-24T158-276L81-710Zm279 110v440h280v-160h160v-280H360Zm220 220Z"
            /></svg
          >
        </button>
        <button
          class="tab"
          class:active={searchMode === 'threads'}
          onclick={() => {
            searchMode = 'threads';
            performSearch();
          }}
        >
          {$t('search.threads')}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="1rem"
            viewBox="0 -960 960 960"
            width="1rem"
            fill="currentColor"
            ><path
              d="M600-80v-100L320-320H120v-240h172l108-124v-196h240v240H468L360-516v126l240 120v-50h240v240H600ZM480-720h80v-80h-80v80ZM200-400h80v-80h-80v80Zm480 240h80v-80h-80v80ZM520-760ZM240-440Zm480 240Z"
            /></svg
          >
        </button>
      </div>

      <button
        class="fuzzy-btn"
        class:active={isFuzzy}
        onclick={() => (isFuzzy = !isFuzzy)}
        title={isFuzzy ? $t('search.fuzzy_on') : $t('search.fuzzy_off')}
      >
        {$t('search.fuzzy')}
      </button>
    </div>

    <div class="input-wrapper">
      <div class="search-icon">
        <svg
          viewBox="0 0 24 24"
          width="18"
          height="18"
          stroke="currentColor"
          stroke-width="2"
          fill="none"
        >
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
      </div>

      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        bind:value={query}
        oninput={onInput}
        placeholder={searchMode === 'notes'
          ? $t('search.start_typing')
          : $t('search.start_typing_threads')}
        spellcheck="false"
        autofocus
      />
    </div>
  </header>

  <main class="results-area" class:loading={isSearching}>
    {#if isSearching && results.length === 0}
      <div class="status-view">
        <div class="spinner"></div>
        <p>{$t('search.searching')}</p>
      </div>
    {:else if results.length > 0}
      {#if searchMode === 'notes'}
        <NoteSearchResults
          results={results as SearchResult[]}
          {nav}
          {query}
          onSelect={selectResult}
        />
      {:else}
        <ThreadSearchResults
          results={results as ThreadSearchResult[]}
          {nav}
          onSelect={selectThread}
        />
      {/if}
    {:else if query.trim().length > 0}
      <div class="status-view">
        <p class="muted">{$t('search.no_results')}</p>
      </div>
    {:else}
      <div class="status-view empty">
        <div class="empty-icon">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="5rem"
            viewBox="0 -960 960 960"
            width="5rem"
            fill="currentColor"
            ><path
              d="M120-240v-80h480v80H120Zm0-200v-80h720v80H120Zm0-200v-80h720v80H120Z"
            /></svg
          >
        </div>
      </div>
    {/if}
  </main>

  <ModalFooter
    shortcuts={[
      { label: $t('search.footer.navigate'), key: '↑↓' },
      { label: $t('search.footer.open'), key: 'Enter' },
      {
        label: $t('search.footer.mode'),
        action: 'toggleSearchMode',
      },
      {
        label: $t('search.footer.fuzzy'),
        action: 'toggleFuzzy',
      },
      { label: $t('search.footer.close'), key: 'Esc' },
    ]}
    count={results.length}
    countLabel={$t('search.results_count', { count: results.length })}
  />
</div>

<style>
  .search-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background-color: var(--bg-main);
    overflow: hidden;
    box-shadow:
      0 10px 25px -5px rgba(0, 0, 0, 0.3),
      0 8px 10px -6px rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
  }

  .search-header {
    padding: 1rem;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .mode-tabs {
    display: flex;
    background-color: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .mode-tabs button {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.5rem;
    font-weight: 800;
  }

  .tab {
    padding: 0.4rem 0.8rem;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
    user-select: none;
  }

  .tab:hover {
    color: var(--text-main);
    background-color: var(--bg-surface);
  }

  .tab.active {
    background-color: var(--accent);
    color: var(--accent-text);
  }

  .fuzzy-btn {
    padding: 0.4rem 0.8rem;
    background: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 0.4rem;
    color: var(--text-muted);
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
    user-select: none;
    font-weight: 800;
  }

  .fuzzy-btn:hover {
    border-color: var(--accent);
    color: var(--text-main);
  }

  .fuzzy-btn.active {
    background-color: var(--accent);
    border-color: var(--accent);
    color: var(--accent-text);
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background-color: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0 0.75rem;
    transition:
      border-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      box-shadow 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .input-wrapper:focus-within {
    background-color: color-mix(in srgb, var(--accent), transparent 80%);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent), transparent 80%);
  }

  .search-icon {
    color: var(--text-muted);
    display: flex;
    align-items: center;
  }

  .input-wrapper input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-main);
    padding: 0.75rem 0;
    font-size: 1rem;
    outline: none;
  }

  .results-area {
    flex: 1;
    overflow-y: auto;
    min-height: 300px;
    position: relative;
  }

  .results-area.loading {
    opacity: 0.7;
  }

  .status-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1rem;
    color: var(--text-muted);
  }

  .empty-icon {
    color: var(--border);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border);
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
  }
</style>
