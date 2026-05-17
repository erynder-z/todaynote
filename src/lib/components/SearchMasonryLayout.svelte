<script lang="ts">
  /**
   * Masonry layout for search results.
   * Displays search result excerpts in a grid-like card format with match highlighting.
   */
  import { ListNavigator } from '$lib';
  import type { SearchResult } from '$lib/types/notes';

  let { results, nav, query, onSelect } = $props<{
    results: SearchResult[];
    nav: ListNavigator;
    query: string;
    onSelect: (result: SearchResult) => void;
  }>();

  let containerWidth = $state(0);

  let columnCount = $derived.by(() => {
    if (containerWidth > 1200) return 4;
    if (containerWidth > 800) return 3;
    if (containerWidth > 500) return 2;
    return 1;
  });

  let columns = $derived.by(() => {
    const cols: SearchResult[][] = Array.from(
      { length: columnCount },
      () => [],
    );
    for (let i = 0; i < results.length; i++)
      cols[i % columnCount].push(results[i]);
    return cols;
  });

  /**
   * Highlight text based on indices and query
   */
  const highlight = (text: string, indices: number[], query: string) => {
    if (!indices || indices.length === 0) return text;
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
          for (let j = 0; j < queryLower.length; j++) exactIndices.add(i + j);
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
      if (indexSet.has(i)) neededMark = exactIndices.has(i) ? 'exact' : 'fuzzy';

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

  /**
   * Handle keyboard navigation
   */
  export const handleKey = (e: KeyboardEvent) => {
    if (['ArrowRight', 'ArrowLeft', 'ArrowUp', 'ArrowDown'].includes(e.key)) {
      e.preventDefault();
      const currentIndex = nav.index === -1 ? 0 : nav.index;
      let nextIndex = currentIndex;

      if (e.key === 'ArrowLeft') nextIndex = Math.max(0, currentIndex - 1);
      else if (e.key === 'ArrowRight')
        nextIndex = Math.min(results.length - 1, currentIndex + 1);
      else if (e.key === 'ArrowUp')
        nextIndex = Math.max(0, currentIndex - columnCount);
      else if (e.key === 'ArrowDown')
        nextIndex = Math.min(results.length - 1, currentIndex + columnCount);

      nav.setIndex(nextIndex, 'keyboard');
      return true;
    }
    return nav.handleKey(e);
  };
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="search-masonry-container"
  bind:clientWidth={containerWidth}
  onmouseleave={() => nav.reset()}
>
  <div class="columns-wrapper">
    {#each columns as column}
      <div class="column-lane">
        {#each column as result}
          {@const globalIdx = results.indexOf(result)}
          <button
            class="result-card"
            class:selected={globalIdx === nav.index}
            onclick={() => onSelect(result)}
            onmouseenter={() => {
              if (nav.shouldIgnoreMouseEnter()) return;
              nav.setIndex(globalIdx, 'mouse');
            }}
          >
            <div class="card-header">
              <span class="note-name">{result.formattedName}</span>
              <span class="ln">L{result.lineNumber + 1}</span>
            </div>
            <div class="card-content">
              <p class="excerpt">
                {@html highlight(result.excerpt, result.indices, query)}
              </p>
            </div>
          </button>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .search-masonry-container {
    width: 100%;
    padding: 1.5rem;
  }

  .columns-wrapper {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }

  .column-lane {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
  }

  .result-card {
    display: flex;
    flex-direction: column;
    padding: 1rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    text-align: left;
    cursor: pointer;
    width: 100%;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .result-card:hover {
    border-color: var(--text-muted);
    transform: translateY(-1px);
  }

  .result-card.selected {
    border-color: var(--accent);
    background-color: color-mix(in srgb, var(--accent), transparent 92%);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .note-name {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 4;
    line-clamp: 4;
    -webkit-box-orient: vertical;
    overflow: hidden;
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
</style>
