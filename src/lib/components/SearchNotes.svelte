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
  import { notesService } from '$lib/utils/notes';
  import { ListNavigator } from '../stores/listNav.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import { settings } from '../stores/settings.svelte';
  import { toast } from '../stores/toast.svelte';
  import { t } from '../utils/i18n';
  import { useShortcuts } from '../utils/shortcuts';
  import ModalFooter from './ModalFooter.svelte';
  import SearchInput from './SearchInput.svelte';
  import SearchResultsContainer from './SearchResultsContainer.svelte';
  import SearchSidebar from './SearchSidebar.svelte';
  import SearchStatusView from './SearchStatusView.svelte';

  let query = $state('');
  let results = $state<
    SearchResult[] | ThreadSearchResult[] | TagSearchResult[]
  >([]);
  let isSearching = $state(false);
  let searchId = 0;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const nav = new ListNavigator(
    () => results.length,
    (i) => {
      if (settings.searchMode === 'notes') {
        selectResult(results[i] as SearchResult);
      } else if (settings.searchMode === 'threads') {
        selectThread(results[i] as ThreadSearchResult);
      } else if (settings.searchSelectedTag) {
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
      settings.setNotesListLayout(nextLayout);
    },
    toggleFuzzy: () => {
      settings.setSearchIsFuzzy(!settings.searchIsFuzzy);
    },
    toggleSearchMode: () => {
      setSearchMode(
        settings.searchMode === 'notes'
          ? 'threads'
          : settings.searchMode === 'threads'
            ? 'tags'
            : 'notes',
      );
    },
  });

  const setSearchMode = (mode: 'notes' | 'threads' | 'tags') => {
    settings.setSearchMode(mode);
    settings.setSearchSelectedTag(null);
    query = '';
    results = [];
    performSearch();
  };

  /**
   * Performs the search operation based on the current query and mode.
   * Debounced via onInput to prevent excessive API calls.
   */
  const performSearch = async () => {
    const id = ++searchId;

    if (
      !query.trim() &&
      settings.searchMode === 'notes' &&
      !settings.searchSelectedTag
    ) {
      results = [];
      isSearching = false;
      nav.reset();
      return;
    }

    isSearching = true;
    try {
      const { searchMode, searchIsFuzzy, searchSelectedTag } = settings;

      const searchActions = {
        notes: () => notesService.searchNotes(query, searchIsFuzzy),
        threads: () => notesService.searchThreads(query, searchIsFuzzy),
        tags: () =>
          searchSelectedTag
            ? notesService.searchNotesByTag(
                searchSelectedTag,
                query,
                searchIsFuzzy,
              )
            : notesService.searchTags(query, searchIsFuzzy),
      };

      const data = await searchActions[searchMode]();

      if (id === searchId) results = data;
    } catch (error) {
      console.error('Search error:', error);
      if (id === searchId) results = [];
    } finally {
      if (id === searchId) {
        isSearching = false;
        nav.reset();
      }
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
    const content = await notesService.readNoteContent(path);
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
    const aggregation = await notesService.aggregateThread(thread.name);
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
    settings.setSearchSelectedTag(tag.name);
    query = '';
    results = [];
    performSearch();
  };

  /**
   * Clears the current tag filter and returns to the full tag list.
   */
  const clearTagFilter = () => {
    settings.setSearchSelectedTag(null);
    query = '';
    results = [];
    performSearch();
  };

  /**
   * Handles global keydown events for list navigation.
   */
  const handleKeydown = (e: KeyboardEvent) => {
    if (results.length === 0) return;

    if (shouldHandleMasonryKey(e)) return;
    nav.handleKey(e);
  };

  /**
   * Handles masonry layout key events if applicable
   * Returns true if the event was handled, false otherwise
   */
  const shouldHandleMasonryKey = (e: KeyboardEvent) => {
    return (
      settings.notesListLayout === 'masonry' &&
      masonryLayout &&
      masonryLayout.handleKey(e)
    );
  };

  $effect(() => {
    // Re-run search if fuzzy mode changes
    if (settings.searchIsFuzzy !== undefined) performSearch();
  });

  // Initial search for non-notes modes to populate browser views
  $effect(() => {
    if (settings.searchMode !== 'notes' && query === '') {
      performSearch();
    }
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="search-container" onkeydown={handleKeydown}>
  <div class="control-center">
    <SearchSidebar
      searchMode={settings.searchMode}
      selectedTag={settings.searchSelectedTag}
      onModeChange={setSearchMode}
    />

    <main class="main-content">
      <div class="toolbar">
        <SearchInput
          bind:query
          isFuzzy={settings.searchIsFuzzy}
          searchMode={settings.searchMode}
          selectedTag={settings.searchSelectedTag}
          {onInput}
          onClearTag={clearTagFilter}
          onClearQuery={() => {
            query = '';
            performSearch();
          }}
          onToggleFuzzy={() =>
            settings.setSearchIsFuzzy(!settings.searchIsFuzzy)}
        />
      </div>

      <div class="results-area" class:loading={isSearching}>
        {#if results.length > 0}
          <SearchResultsContainer
            {results}
            searchMode={settings.searchMode}
            selectedTag={settings.searchSelectedTag}
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
            selectedTag={settings.searchSelectedTag}
          />
        {/if}
      </div>

      <ModalFooter
        shortcuts={[
          ...(settings.notesListLayout === 'masonry'
            ? [{ label: $t('browser.navigate'), key: '🠝🠟 🠜🠞' }]
            : [{ label: $t('browser.navigate'), key: '🠝🠟' }]),
          { label: $t('search.footer.open'), key: '↵' },
          {
            label: $t('shortcuts.search.toggle_fuzzy'),
            action: 'toggleFuzzy',
          },
          {
            label: $t('shortcuts.action.toggle_note_browser_layout'),
            action: 'toggleNoteBrowserLayout',
          },
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

  .toolbar {
    padding: 1rem 2rem;
    background-color: var(--bg-surface);
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .results-area {
    flex: 1;
    overflow: hidden;
    position: relative;
    display: flex;
    flex-direction: column;
  }
</style>
