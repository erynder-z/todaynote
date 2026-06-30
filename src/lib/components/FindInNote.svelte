<script lang="ts">
  /**
   * FindInNote component providing find-in-page browser-like functionality.
   * Toggleable via the "primary + F" keyboard shortcut.
   */
  import { onDestroy, tick } from 'svelte';
  import { fly } from 'svelte/transition';
  import { t } from '$lib/utils/i18n';
  import { sessionState } from '../stores/sessionState.svelte';
  import { DOMObserver } from '../utils/domObserver';
  import { FindInElement } from '../utils/findInElement';
  import { Highlighter } from '../utils/highlighter';
  import { useShortcuts } from '../utils/shortcuts';

  let inputEl = $state<HTMLInputElement | null>(null);
  let searchQuery = $state('');
  let searchResults = $state<Range[]>([]);
  let currentIndex = $state(0);

  const observer = new DOMObserver();
  const highlightService = new Highlighter();
  const isSearchable = $derived.by(() => {
    const isNoteShown =
      sessionState.todayNoteContent !== null &&
      sessionState.activePopup === null;

    const isThreadAggregationShown =
      sessionState.activePopup === 'threadAggregation';

    return isNoteShown || isThreadAggregationShown;
  });

  /**
   * Determine active target element for searching.
   */
  const getTargetElement = (): HTMLElement | null => {
    if (sessionState.activePopup === 'threadAggregation')
      return document.querySelector('.aggregation-container');

    return document.querySelector('.editor-main');
  };

  const findService = new FindInElement(getTargetElement);

  /**
   * Close the search panel.
   */
  const close = () => {
    sessionState.showFindInNote = false;
    searchQuery = '';
  };

  /**
   *  Component keydown handler.
   */
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      close();
      e.preventDefault();
      e.stopPropagation();
      return;
    }

    if (e.key === 'Enter') {
      e.shiftKey ? prevMatch() : nextMatch();
      e.preventDefault();
      e.stopPropagation();
    }
  };

  /**
   * Apply highlights and scroll active match into view.
   */
  const updateHighlights = () => {
    highlightService.applyHighlights(searchResults, currentIndex);
  };

  const scrollToActiveMatch = () => {
    const activeRange = searchResults[currentIndex];
    if (!activeRange) return;

    activeRange.startContainer.parentElement?.scrollIntoView({
      block: 'center',
      behavior: 'smooth',
    });
  };

  /**
   * Perform a new search.
   */
  const performSearch = () => {
    highlightService.clearHighlights();

    searchResults = findService.search(searchQuery);

    if (searchResults.length === 0) {
      currentIndex = 0;
      return;
    }

    if (currentIndex >= searchResults.length) currentIndex = 0;

    updateHighlights();
    scrollToActiveMatch();
  };

  /**
   * Start observing DOM changes.
   */
  const setupObserver = () => {
    const target = getTargetElement();

    if (!target) {
      observer.disconnect();
      return;
    }

    observer.observe(target, performSearch);
  };

  /**
   * Navigate to next match.
   */
  const nextMatch = () => {
    if (searchResults.length === 0) return;

    currentIndex = (currentIndex + 1) % searchResults.length;
    updateHighlights();
    scrollToActiveMatch();
  };

  /**
   * Navigate to previous match.
   */
  const prevMatch = () => {
    if (searchResults.length === 0) return;

    currentIndex =
      (currentIndex - 1 + searchResults.length) % searchResults.length;

    updateHighlights();
    scrollToActiveMatch();
  };

  /**
   * Focus and select the search input.
   */
  const focusSearchInput = () => {
    inputEl?.focus();
    inputEl?.select();
  };

  /**
   * Resets the search state.
   */
  const resetSearch = () => {
    highlightService.clearHighlights();
    searchResults = [];
    currentIndex = 0;
    observer.disconnect();
  };

  useShortcuts({
    closePopup: () => {
      if (!sessionState.showFindInNote) return false;

      close();
      return true;
    },
  });

  /**
   * Focus the search input when the panel opens.
   */
  $effect(() => {
    if (!sessionState.showFindInNote) return;
    focusSearchInput();
  });

  /**
   * Keep search results synchronized with the current view.
   */
  $effect(() => {
    if (!sessionState.showFindInNote || !isSearchable) {
      resetSearch();
      return;
    }

    sessionState.activePopup;
    sessionState.todayNotePath;
    searchQuery;

    tick().then(() => {
      performSearch();
      setupObserver();
    });
  });

  onDestroy(() => {
    resetSearch();
    highlightService.cleanup();
  });
</script>

{#if sessionState.showFindInNote}
  <div class="search-in-view" transition:fly={{ y: -15, duration: 150 }}>
    <div class="search-icon">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.2rem"
        viewBox="0 -960 960 960"
        width="1.2rem"
        fill="currentColor"
        ><path
          d="m590-160 80 80H240q-33 0-56.5-23.5T160-160v-640q0-33 23.5-56.5T240-880h360l200 240v480q0 20-8.5 36.5T768-96L560-302q-17 11-37 16.5t-43 5.5q-66 0-113-47t-47-113q0-66 47-113t113-47q66 0 113 47t47 113q0 23-5.5 43T618-360l102 104v-356L562-800H240v640h350Zm-53.5-223.5Q560-407 560-440t-23.5-56.5Q513-520 480-520t-56.5 23.5Q400-473 400-440t23.5 56.5Q447-360 480-360t56.5-23.5ZM480-440Zm0 0Z"
        /></svg
      >
    </div>

    <input
      bind:this={inputEl}
      type="text"
      placeholder={$t('find_in_note.placeholder')}
      bind:value={searchQuery}
      onkeydown={handleKeyDown}
      class="search-input"
    />

    <div class="search-status">
      {searchResults.length > 0
        ? `${currentIndex + 1} / ${searchResults.length}`
        : '0 / 0'}
    </div>

    <div class="search-actions">
      <button
        class="action-btn"
        onclick={prevMatch}
        title={$t('find_in_note.prev')}
        aria-label={$t('find_in_note.prev')}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
        >
          <path d="m296-360-56-56 240-240 240 240-56 56-184-184-184 184Z" />
        </svg>
      </button>

      <button
        class="action-btn"
        onclick={nextMatch}
        title={$t('find_in_note.next')}
        aria-label={$t('find_in_note.next')}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
        >
          <path d="M480-344 240-584l56-56 184 184 184-184 56 56-240 240Z" />
        </svg>
      </button>

      <div class="divider"></div>

      <button
        class="close-btn"
        onclick={close}
        title={$t('find_in_note.close')}
        aria-label={$t('find_in_note.close')}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
        >
          <path
            d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
          />
        </svg>
      </button>
    </div>
  </div>
{/if}

<style>
  .search-in-view {
    position: fixed;
    bottom: 1rem;
    left: 3.5rem;
    z-index: 3000;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background-color: color-mix(in srgb, var(--bg-surface), transparent 15%);
    backdrop-filter: blur(0.5rem);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0.375rem 0.625rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
    min-width: 20rem;
  }

  .search-icon {
    color: var(--text-muted);
    display: flex;
    align-items: center;
    padding-left: 0.25rem;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-main);
    font-size: 0.9rem;
    font-family: inherit;
    outline: none;
    padding: 0.25rem 0;
    min-width: 8rem;
  }

  .search-status {
    font-size: 0.8rem;
    color: var(--text-muted);
    user-select: none;
    padding: 0 0.5rem;
    white-space: nowrap;
  }

  .search-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .action-btn,
  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 0.25rem;
    border-radius: 0.25rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .action-btn:hover {
    color: var(--text-main);
    background-color: color-mix(in srgb, var(--text-muted), transparent 90%);
  }

  .close-btn:hover {
    color: var(--error, #ef4444);
    background-color: color-mix(
      in srgb,
      var(--error, #ef4444),
      transparent 90%
    );
  }

  .divider {
    width: 1px;
    height: 1rem;
    background-color: var(--border);
    margin: 0 0.25rem;
  }

  /* CSS Custom Highlight Styling */
  :global(::highlight(search-results)) {
    background-color: var(--search-match-bg, transparent);
    color: inherit;
  }

  :global(::highlight(current-search-result)) {
    background-color: var(--search-current-bg, transparent);
    color: var(--search-current-fg, inherit);
  }
</style>
