<script lang="ts">
  /**
   * Component for displaying a list of note search results.
   * Handles highlighting of search matches in excerpts.
   */
  import type { ListNavigator } from '$lib';
  import type { SearchResult } from '$lib/interfaces/notes';

  let { results, nav, query, onSelect } = $props<{
    results: SearchResult[];
    nav: ListNavigator;
    query: string;
    onSelect: (result: SearchResult) => void;
  }>();

  /**
   * Highlights search terms within a text string using HTML <mark> tags.
   * Supports both exact and fuzzy matches with different visual styles.
   */
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
</script>

<div class="results-list">
  {#each results as result, i}
    <button
      class="result-item"
      class:selected={i === nav.index}
      onclick={() => onSelect(result)}
      onmouseenter={() => {
        if (nav.shouldIgnoreMouseEnter()) return;
        nav.setIndex(i, 'mouse');
      }}
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

<style>
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
    transition: background-color 0.1s cubic-bezier(0.2, 0, 0, 1);
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
</style>
