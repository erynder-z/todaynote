<script lang="ts">
  /**
   * Enhanced note search component with command-palette aesthetics
   * and full keyboard navigation.
   */
  import { ListNavigator, sessionState, settings, t } from '$lib';
  import { inputManager } from '$lib/stores/input.svelte';
  import type { SearchResult } from '$lib/types/notes';
  import { readNoteContent, searchNotes } from '$lib/utils/notes';

  let query = $state('');
  let isFuzzy = $state(true);
  let results = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const nav = new ListNavigator(
    () => results.length,
    (i) => selectResult(results[i]),
  );

  $effect(() => {
    return inputManager.registerAction('toggleFuzzy', () => {
      isFuzzy = !isFuzzy;
    });
  });

  const performSearch = async () => {
    if (query.trim().length === 0) {
      results = [];
      nav.reset();
      return;
    }

    isSearching = true;
    const searchResults = await searchNotes(query, isFuzzy);
    results = searchResults;
    isSearching = false;
    nav.reset();
  };

  const onInput = () => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(performSearch, 200);
  };

  const selectResult = async (result: SearchResult) => {
    if (!settings.notesFolder || !result) return;
    const path = `${settings.notesFolder}/${result.filename}`;
    const content = await readNoteContent(path);
    if (content !== null) {
      sessionState.todayNotePath = path;
      sessionState.todayNoteContent = content;
      sessionState.activePopup = null;
    }
  };

  const highlight = (text: string, indices: number[], query: string) => {
    if (!indices || indices.length === 0) {
      return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
    }

    const chars = Array.from(text);
    const queryChars = Array.from(query.trim());
    const exactIndices = new Set<number>();

    if (queryChars.length > 0) {
      const textLower = chars.map((c) => c.toLowerCase());
      const queryLower = queryChars.map((c) => c.toLowerCase());

      for (let i = 0; i <= textLower.length - queryLower.length; i++) {
        let match = true;
        for (let j = 0; j < queryLower.length; j++) {
          if (textLower[i + j] !== queryLower[j]) {
            match = false;
            break;
          }
        }
        if (match)
          for (let j = 0; j < queryLower.length; j++) {
            exactIndices.add(i + j);
          }
      }
    }

    const indexSet = new Set(indices);
    let result = '';
    let currentMark: 'exact' | 'fuzzy' | null = null;

    for (let i = 0; i < chars.length; i++) {
      const char = chars[i]
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');

      let neededMark: 'exact' | 'fuzzy' | null = null;
      if (indexSet.has(i)) {
        neededMark = exactIndices.has(i) ? 'exact' : 'fuzzy';
      }

      if (neededMark !== currentMark) {
        if (currentMark) result += '</mark>';
        if (neededMark) result += `<mark class="${neededMark}">`;
        currentMark = neededMark;
      }
      result += char;
    }
    if (currentMark) result += '</mark>';

    return result;
  };

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
        placeholder={$t('search.start_typing')}
        spellcheck="false"
        autofocus
      />

      <button
        class="mode-badge"
        class:active={isFuzzy}
        onclick={() => (isFuzzy = !isFuzzy)}
        title={$t('search.fuzzy')}
      >
        <span>Fuzzy</span>
      </button>
    </div>
  </header>

  <main class="results-area" class:loading={isSearching}>
    {#if isSearching && results.length === 0}
      <div class="status-view">
        <div class="spinner"></div>
        <p>{$t('search.searching')}</p>
      </div>
    {:else if results.length > 0}
      <div class="results-list">
        {#each results as result, i}
          <button
            class="result-item"
            class:selected={i === nav.index}
            onclick={() => selectResult(result)}
            onmouseenter={() => (nav.index = i)}
          >
            <div class="result-meta">
              <span class="date">{result.formattedName}</span>
              <span class="ln">L{result.lineNumber + 1}</span>
            </div>
            <div class="result-content">
              <p class="excerpt">
                {@html highlight(result.excerpt, result.indices, query)}
              </p>
            </div>
          </button>
        {/each}
      </div>
    {:else if query.trim().length > 0}
      <div class="status-view">
        <p class="muted">{$t('search.no_results')}</p>
      </div>
    {:else}
      <div class="status-view empty">
        <div class="empty-icon">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="10rem"
            viewBox="0 -960 960 960"
            width="10rem"
            fill="currentColor"
            ><path
              d="M784-120 532-372q-30 24-69 38t-83 14q-109 0-184.5-75.5T120-580q0-109 75.5-184.5T380-840q109 0 184.5 75.5T640-580q0 44-14 83t-38 69l252 252-56 56ZM380-400q75 0 127.5-52.5T560-580q0-75-52.5-127.5T380-760q-75 0-127.5 52.5T200-580q0 75 52.5 127.5T380-400Z"
            /></svg
          >
        </div>
      </div>
    {/if}
  </main>

  <footer class="search-footer">
    <div class="shortcuts">
      <span>{$t('search.footer.navigate')}</span> <span class="key">↑↓</span>
      <span>{$t('search.footer.open')}</span> <span class="key">Enter</span>
      <span>{$t('search.footer.fuzzy')}</span>
      <span class="key"
        >{inputManager.primaryLabel}+{inputManager.secondaryLabel}+F</span
      >
      <span>{$t('search.footer.close')}</span> <span class="key">Esc</span>
    </div>
    <div class="count">
      {$t('search.results_count', { count: results.length })}
    </div>
  </footer>
</div>

<style>
  .search-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 60dvh;
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
      border-color 0.2s,
      box-shadow 0.2s;
  }

  .input-wrapper:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent), transparent 80%);
  }

  .search-icon {
    color: var(--text-muted);
    display: flex;
    align-items: center;
  }

  input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-main);
    padding: 0.75rem 0;
    font-size: 1rem;
    outline: none;
  }

  .mode-badge {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-muted);
    font-size: 0.7rem;
    font-weight: 700;
    text-transform: uppercase;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    user-select: none;
  }

  .mode-badge.active {
    background: var(--accent);
    color: var(--accent-text);
    border-color: var(--accent);
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

  .results-list {
    display: flex;
    flex-direction: column;
  }

  .result-item {
    display: flex;
    flex-direction: column;
    padding: 0.75rem 1rem;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    text-align: left;
    cursor: pointer;
    width: 100%;
    transition: background-color 0.1s;
  }

  .result-item:last-child {
    border-bottom: none;
  }

  .result-item.selected {
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
  }

  .result-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .date {
    font-weight: 600;
    font-size: 0.8rem;
    color: var(--accent);
  }

  .ln {
    font-size: 0.7rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .excerpt {
    font-size: 0.9rem;
    color: var(--text-main);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.4;
  }

  :global(.excerpt mark.exact) {
    background-color: var(--accent);
    color: var(--accent-text);
    padding: 0 2px;
    border-radius: 2px;
    font-weight: 600;
  }

  :global(.excerpt mark.fuzzy) {
    background-color: color-mix(in srgb, var(--accent), transparent 70%);
    color: var(--text-main);
    padding: 0 2px;
    border-radius: 2px;
    border-bottom: 2px solid var(--accent);
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

  .search-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: var(--bg-surface);
    border-top: 1px solid var(--border);
    font-size: 0.75rem;
    color: var(--text-main);
  }

  .shortcuts {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .key {
    background-color: var(--bg-main);
    border: 1px solid var(--border);
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    color: var(--text-main);
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .muted {
    font-style: italic;
  }
</style>
