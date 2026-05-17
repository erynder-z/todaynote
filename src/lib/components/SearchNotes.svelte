<script lang="ts">
  /**
   * Note search layout component.
   * Orchestrates sub-components for sidebar, input, and results.
   */

  import type {
    SearchResult,
    TagSearchResult,
    ThreadSearchResult,
  } from '$lib/interfaces/notes';
  import { inputManager } from '$lib/stores/input.svelte';
  import {
    aggregateThread,
    readNoteContent,
    searchNotes,
    searchNotesByTag,
    searchTags,
    searchThreads,
  } from '$lib/utils/notes';
  import { ListNavigator } from '../stores/listNav.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import { settings } from '../stores/settings.svelte';
  import { toast } from '../stores/toast.svelte';
  import { t } from '../utils/i18n';
  import { useShortcuts } from '../utils/shortcuts';
  import LayoutToolbar from './LayoutToolbar.svelte';
  import ModalFooter from './ModalFooter.svelte';
  import SearchInput from './SearchInput.svelte';
  import SearchResultsContainer from './SearchResultsContainer.svelte';
  import SearchSidebar from './SearchSidebar.svelte';
  import SearchStatusView from './SearchStatusView.svelte';

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
      <LayoutToolbar onLayoutChange={setLayout} />

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
        {#if results.length > 0}
          <SearchResultsContainer
            {results}
            {searchMode}
            {selectedTag}
            {query}
            {nav}
            bind:masonryLayout
            onSelectNote={selectResult}
            onSelectThread={selectThread}
            onSelectTag={selectTag}
          />
        {:else}
          <SearchStatusView
            {isSearching}
            hasResults={results.length > 0}
            {query}
            {selectedTag}
          />
        {/if}
      </div>

      <ModalFooter
        shortcuts={[
          ...(settings.notesListLayout === 'masonry'
            ? [{ label: $t('browser.navigate'), key: '↑↓←→' }]
            : [{ label: $t('browser.navigate'), key: '↑↓' }]),
          { label: $t('search.footer.open'), key: 'Enter' },
          { label: $t('search.footer.clear'), key: 'Delete' },
          {
            label: $t('shortcuts.search.toggle_fuzzy'),
            action: 'toggleFuzzy',
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

  .results-area {
    flex: 1;
    overflow: hidden;
    position: relative;
    display: flex;
    flex-direction: column;
  }
</style>
