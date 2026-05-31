<script lang="ts">
  /**
   * Container component that renders the appropriate result component
   * based on search mode, layout, and selected tag.
   */
  import type {
    SearchResult,
    TagSearchResult,
    ThreadSearchResult,
  } from '$lib/interfaces/notes';
  import type { SearchResultsContainerProps } from '$lib/interfaces/ui';
  import { formatNoteName } from '$lib/utils/notes';
  import { settings } from '../stores/settings.svelte';
  import { locale, t } from '../utils/i18n';
  import ListLayout from './ListLayout.svelte';
  import MasonryLayout from './MasonryLayout.svelte';

  let {
    results,
    searchMode,
    selectedTag,
    query,
    nav,
    masonryLayout = $bindable(null),
    onSelectNote,
    onSelectThread,
    onSelectTag,
  }: SearchResultsContainerProps = $props();

  /**
   * Highlight text based on indices and query
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

{#snippet noteListSnippet(result: SearchResult)}
  <div class="list-note-item">
    <div class="result-meta">
      <span class="date">{formatNoteName(result.filename, $locale)}</span>
      <span class="ln">L{result.lineNumber + 1}</span>
    </div>
    <div class="result-content">
      <p class="excerpt">
        {@html highlight(result.excerpt, result.indices, query)}
      </p>
    </div>
  </div>
{/snippet}

{#snippet noteMasonrySnippet(result: SearchResult)}
  <div class="masonry-note-item">
    <div class="card-header">
      <span class="note-name">{formatNoteName(result.filename, $locale)}</span>
      <span class="ln">L{result.lineNumber + 1}</span>
    </div>
    <div class="card-content">
      <p class="excerpt">
        {@html highlight(result.excerpt, result.indices, query)}
      </p>
    </div>
  </div>
{/snippet}

{#snippet threadTagListSnippet(item: ThreadSearchResult | TagSearchResult)}
  <div class="list-thread-tag-item">
    <div class="item-icon">
      {#if searchMode === 'tags'}
        <span class="hash">#</span>
      {:else}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1rem"
          viewBox="0 -960 960 960"
          width="1rem"
          fill="currentColor"
          ><path
            d="M600-80v-100L320-320H120v-240h172l108-124v-196h240v240H468L360-516v126l240 120v-50h240v240H600Z"
          /></svg
        >
      {/if}
    </div>
    <span class="item-name">{item.name}</span>
    <span class="item-count">
      {$t(
        item.noteCount === 1
          ? 'search.note_count_single'
          : 'search.note_count_multiple',
        { count: item.noteCount },
      )}
    </span>
  </div>
{/snippet}

{#snippet threadTagMasonrySnippet(item: ThreadSearchResult | TagSearchResult)}
  <div class="masonry-thread-tag-item" class:tag-item={searchMode === 'tags'}>
    <div class="card-icon">
      {#if searchMode === 'tags'}
        <span class="hash">#</span>
      {:else}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1rem"
          viewBox="0 -960 960 960"
          width="1rem"
          fill="currentColor"
          ><path
            d="M600-80v-100L320-320H120v-240h172l108-124v-196h240v240H468L360-516v126l240 120v-50h240v240H600Z"
          /></svg
        >
      {/if}
    </div>
    <span class="item-name">{item.name}</span>
    <div class="item-count">
      {$t(
        item.noteCount === 1
          ? 'search.note_count_single'
          : 'search.note_count_multiple',
        { count: item.noteCount },
      )}
    </div>
  </div>
{/snippet}

<div class="results-scroll">
  {#if searchMode === 'notes' || selectedTag}
    {#if settings.notesListLayout === 'masonry'}
      <MasonryLayout
        bind:this={masonryLayout}
        items={results as SearchResult[]}
        {nav}
        onSelect={onSelectNote}
        itemSnippet={noteMasonrySnippet}
      />
    {:else}
      <ListLayout
        items={results as SearchResult[]}
        {nav}
        onSelect={onSelectNote}
        itemSnippet={noteListSnippet}
      />
    {/if}
  {:else if settings.notesListLayout === 'masonry'}
    <MasonryLayout
      bind:this={masonryLayout}
      items={results as (ThreadSearchResult | TagSearchResult)[]}
      {nav}
      onSelect={(item) =>
        searchMode === 'threads'
          ? onSelectThread(item as ThreadSearchResult)
          : onSelectTag(item as TagSearchResult)}
      itemSnippet={threadTagMasonrySnippet}
    />
  {:else}
    <ListLayout
      items={results as (ThreadSearchResult | TagSearchResult)[]}
      {nav}
      onSelect={(item) =>
        searchMode === 'threads'
          ? onSelectThread(item as ThreadSearchResult)
          : onSelectTag(item as TagSearchResult)}
      itemSnippet={threadTagListSnippet}
    />
  {/if}
</div>

<style>
  .results-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }

  .list-note-item {
    display: flex;
    flex-direction: column;
    padding: 0.75rem 1rem;
    width: 100%;
  }

  .list-thread-tag-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.85rem 1.5rem;
    width: 100%;
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

  .item-icon {
    color: var(--accent);
    opacity: 0.7;
    display: flex;
    align-items: center;
    min-width: 1.2rem;
  }

  .item-name {
    flex: 1;
    font-weight: 600;
    color: var(--text-main);
  }

  .item-count {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .hash {
    font-weight: 800;
    font-size: 1.1rem;
  }

  .masonry-note-item {
    padding: 1rem;
    width: 100%;
  }

  .masonry-thread-tag-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 1.5rem 1rem;
    width: 100%;
    gap: 0.5rem;
    text-align: center;
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

  .card-icon {
    color: var(--accent);
    opacity: 0.7;
    display: flex;
    align-items: center;
    justify-content: center;
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
