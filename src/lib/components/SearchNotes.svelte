<script lang="ts">
  /**
   * Enhanced note search component with "Control Center" aesthetics.
   * Orchestrates sub-components for sidebar, input, and results.
   */
  import {
    ListNavigator,
    ModalFooter,
    NoteSearchResults,
    SearchInput,
    SearchMasonryLayout,
    SearchSidebar,
    sessionState,
    settings,
    TagSearchResults,
    ThreadSearchResults,
    ThreadTagMasonryLayout,
    t,
    toast,
    useShortcuts,
  } from '$lib';
  import { inputManager } from '$lib/stores/input.svelte';
  import type {
    SearchResult,
    TagSearchResult,
    ThreadSearchResult,
  } from '$lib/types/notes';
  import {
    aggregateThread,
    readNoteContent,
    searchNotes,
    searchNotesByTag,
    searchTags,
    searchThreads,
  } from '$lib/utils/notes';

  let query = $state('');
  let isFuzzy = $state(true);
  let searchMode = $state<'notes' | 'threads' | 'tags'>('notes');
  let results = $state<
    SearchResult[] | ThreadSearchResult[] | TagSearchResult[]
  >([]);
  let isSearching = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let selectedTag = $state<string | null>(null);

  const nav = new ListNavigator(
    () => results.length,
    (i) => {
      if (searchMode === 'notes') {
        selectResult(results[i] as SearchResult);
      } else if (searchMode === 'threads') {
        selectThread(results[i] as ThreadSearchResult);
      } else if (selectedTag) {
        selectResult(results[i] as SearchResult);
      } else {
        selectTag(results[i] as TagSearchResult);
      }
    },
  );

  let masonryLayout: { handleKey: (e: KeyboardEvent) => boolean } | null =
    $state(null);

  useShortcuts({
    toggleNoteBrowserLayout: () => {
      const nextLayout =
        settings.notesListLayout === 'list' ? 'masonry' : 'list';
      setLayout(nextLayout);
    },
  });

  $effect(() => {
    return inputManager.registerActions({
      toggleFuzzy: () => {
        isFuzzy = !isFuzzy;
      },
      toggleSearchMode: () => {
        setSearchMode(
          searchMode === 'notes'
            ? 'threads'
            : searchMode === 'threads'
              ? 'tags'
              : 'notes',
        );
      },
    });
  });

  const setSearchMode = (mode: 'notes' | 'threads' | 'tags') => {
    searchMode = mode;
    selectedTag = null;
    query = '';
    results = [];
    performSearch();
  };

  const setLayout = (layout: 'list' | 'masonry') => {
    settings.save({
      notesFolder: settings.notesFolder,
      locale: settings.locale,
      theme: settings.theme,
      rememberWindowSize: settings.rememberWindowSize,
      notesListLayout: layout,
    });
  };

  /**
   * Performs the search operation based on the current query and mode.
   * Debounced via onInput to prevent excessive API calls.
   */
  const performSearch = async () => {
    if (query.trim().length === 0 && searchMode === 'notes' && !selectedTag) {
      results = [];
      nav.reset();
      return;
    }

    isSearching = true;
    try {
      if (searchMode === 'notes') {
        results = await searchNotes(query, isFuzzy);
      } else if (searchMode === 'threads') {
        results = await searchThreads(query, isFuzzy);
      } else if (selectedTag) {
        results = await searchNotesByTag(selectedTag, query, isFuzzy);
      } else {
        results = await searchTags(query, isFuzzy);
      }
    } catch (error) {
      console.error('Search error:', error);
      results = [];
    } finally {
      isSearching = false;
      nav.reset();
    }
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
   * Activates drill-down view for a specific tag.
   */
  const selectTag = (tag: TagSearchResult) => {
    if (!tag) return;
    selectedTag = tag.name;
    query = '';
    results = [];
    performSearch();
  };

  /**
   * Clears the current tag filter and returns to the full tag list.
   */
  const clearTagFilter = () => {
    selectedTag = null;
    query = '';
    results = [];
    performSearch();
  };

  /**
   * Handles global keydown events for list navigation.
   */
  const handleKeydown = (e: KeyboardEvent) => {
    if (results.length > 0) {
      if (settings.notesListLayout === 'masonry' && masonryLayout) {
        if (masonryLayout.handleKey(e)) return;
      }
      nav.handleKey(e);
    }
  };

  $effect(() => {
    // Re-run search if fuzzy mode changes
    if (isFuzzy !== undefined) performSearch();
  });

  // Initial search for non-notes modes to populate browser views
  $effect(() => {
    if (searchMode !== 'notes' && query === '') {
      performSearch();
    }
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="search-container" onkeydown={handleKeydown}>
  <div class="control-center">
    <SearchSidebar {searchMode} {selectedTag} onModeChange={setSearchMode} />

    <main class="main-content">
      <div class="layout-toolbar">
        <div class="toggle-group">
          <button
            class="toggle-btn"
            class:active={settings.notesListLayout === 'list'}
            onclick={() => setLayout('list')}
            title="List View"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              height="1.25rem"
              viewBox="0 -960 960 960"
              width="1.25rem"
              fill="currentColor"
              ><path
                d="M240-240v-120h480v120H240Zm0-200v-120h480v120H240Zm0-200v-120h480v120H240Z"
              /></svg
            >
          </button>
          <button
            class="toggle-btn"
            class:active={settings.notesListLayout === 'masonry'}
            onclick={() => setLayout('masonry')}
            title="Masonry View"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              height="1.25rem"
              viewBox="0 -960 960 960"
              width="1.25rem"
              fill="currentColor"
              ><path
                d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h560q33 0 56.5 23.5T840-760v560q0-33-23.5-56.5T760-120H200Zm0-80h240v-180H200v180Zm320 0h240v-300H520v300ZM200-460h240v-300H200v300Zm320 0h240v-180H520v180Z"
              /></svg
            >
          </button>
        </div>
      </div>
      <SearchInput
        bind:query
        bind:isFuzzy
        {searchMode}
        {selectedTag}
        {onInput}
        onClearTag={clearTagFilter}
        onClearQuery={() => {
          query = '';
          performSearch();
        }}
      />

      <div class="results-area" class:loading={isSearching}>
        {#if isSearching && results.length === 0}
          <div class="status-view">
            <div class="spinner"></div>
            <p>{$t('search.searching')}</p>
          </div>
        {:else if results.length > 0}
          <div class="results-scroll">
            {#if searchMode === 'notes' || selectedTag}
              {#if settings.notesListLayout === 'masonry'}
                <SearchMasonryLayout
                  bind:this={masonryLayout}
                  results={results as SearchResult[]}
                  {nav}
                  {query}
                  onSelect={selectResult}
                />
              {:else}
                <NoteSearchResults
                  results={results as SearchResult[]}
                  {nav}
                  {query}
                  onSelect={selectResult}
                />
              {/if}
            {:else if searchMode === 'threads'}
              {#if settings.notesListLayout === 'masonry'}
                <ThreadTagMasonryLayout
                  bind:this={masonryLayout}
                  results={results as ThreadSearchResult[]}
                  {nav}
                  {searchMode}
                  onSelect={selectThread}
                />
              {:else}
                <ThreadSearchResults
                  results={results as ThreadSearchResult[]}
                  {nav}
                  onSelect={selectThread}
                />
              {/if}
            {:else if settings.notesListLayout === 'masonry'}
              <ThreadTagMasonryLayout
                bind:this={masonryLayout}
                results={results as TagSearchResult[]}
                {nav}
                {searchMode}
                onSelect={selectTag}
              />
            {:else}
              <TagSearchResults
                results={results as TagSearchResult[]}
                {nav}
                onSelect={selectTag}
              />
            {/if}
          </div>
        {:else if query.trim().length > 0 || selectedTag}
          <div class="status-view">
            <p class="muted">{$t('search.no_results')}</p>
          </div>
        {:else}
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
      </div>

      <ModalFooter
        shortcuts={[
          ...(settings.notesListLayout === 'masonry'
            ? [{ label: $t('browser.navigate'), key: '↑↓←→' }]
            : [{ label: $t('browser.navigate'), key: '↑↓' }]),
          { label: $t('search.footer.open'), key: 'Enter' },
          {
            label: $t('search.footer.mode'),
            action: 'toggleSearchMode',
          },
          {
            label: $t('shortcuts.action.toggle_note_browser_layout'),
            action: 'toggleNoteBrowserLayout',
          },
          { label: $t('search.footer.close'), key: 'Esc' },
        ]}
        count={results.length}
        countLabel={$t('search.results_count', { count: results.length })}
      />
    </main>
  </div>
</div>

<style>
  .search-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background-color: var(--bg-main);
    overflow: hidden;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.4);
    border: 1px solid var(--border);
  }

  .control-center {
    display: grid;
    grid-template-columns: 220px 1fr;
    flex: 1;
    overflow: hidden;
  }

  @media (max-width: 768px) {
    .control-center {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr;
    }
  }

  .main-content {
    display: flex;
    flex-direction: column;
    background-color: var(--bg-main);
    overflow: hidden;
  }

  .layout-toolbar {
    display: flex;
    justify-content: flex-end;
    padding: 0.75rem 1.5rem;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
  }

  .toggle-group {
    display: flex;
    background-color: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 2px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.4rem;
    background: none;
    border: none;
    border-radius: 0.35rem;
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      color 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .toggle-btn:hover {
    color: var(--text-main);
  }

  .toggle-btn.active {
    background-color: var(--accent);
    color: white;
  }

  .results-area {
    flex: 1;
    overflow: hidden;
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .results-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }

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
