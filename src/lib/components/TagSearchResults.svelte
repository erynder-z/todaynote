<script lang="ts">
  /**
   * Component for displaying a list of tag search results.
   * Shows tag names and the count of notes they are found in.
   */
  import type { ListNavigator } from '$lib';
  import { t } from '$lib';
  import type { TagSearchResult } from '$lib/interfaces/notes';

  let { results, nav, onSelect } = $props<{
    results: TagSearchResult[];
    nav: ListNavigator;
    onSelect: (result: TagSearchResult) => void;
  }>();
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
        <span class="tag-name">#{result.name}</span>

        <span class="note-count">
          {$t(
            result.noteCount === 1
              ? 'search.note_count_single'
              : 'search.note_count_multiple',
            { count: result.noteCount },
          )}
        </span>
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
    padding: 1rem;
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

  .tag-name {
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--accent);
  }

  .note-count {
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
</style>
