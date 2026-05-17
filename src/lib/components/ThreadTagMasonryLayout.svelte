<script lang="ts">
  /**
   * Masonry layout for Thread and Tag search results.
   * Displays items as adaptive cards with their respective note counts.
   */
  import { ListNavigator, t } from '$lib';
  import type { TagSearchResult, ThreadSearchResult } from '$lib/types/notes';

  let { results, nav, searchMode, onSelect } = $props<{
    results: (ThreadSearchResult | TagSearchResult)[];
    nav: ListNavigator;
    searchMode: 'threads' | 'tags';
    onSelect: (result: ThreadSearchResult | TagSearchResult) => void;
  }>();

  let containerWidth = $state(0);

  let columnCount = $derived.by(() => {
    if (containerWidth > 1200) return 5;
    if (containerWidth > 800) return 4;
    if (containerWidth > 500) return 2;
    return 1;
  });

  let columns = $derived.by(() => {
    const cols: (ThreadSearchResult | TagSearchResult)[][] = Array.from(
      { length: columnCount },
      () => [],
    );
    for (let i = 0; i < results.length; i++) {
      cols[i % columnCount].push(results[i]);
    }
    return cols;
  });

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
  class="masonry-container"
  bind:clientWidth={containerWidth}
  onmouseleave={() => nav.reset()}
>
  <div class="columns-wrapper">
    {#each columns as column}
      <div class="column-lane">
        {#each column as item}
          {@const globalIdx = results.indexOf(item)}
          <button
            class="item-card"
            class:selected={globalIdx === nav.index}
            class:tag-item={searchMode === 'tags'}
            onclick={() => onSelect(item)}
            onmouseenter={() => {
              if (nav.shouldIgnoreMouseEnter()) return;
              nav.setIndex(globalIdx, 'mouse');
            }}
          >
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
          </button>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .masonry-container {
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

  .item-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 1.5rem 1rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    text-align: center;
    cursor: pointer;
    width: 100%;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
    gap: 0.5rem;
  }

  .item-card:hover {
    border-color: var(--text-muted);
    transform: translateY(-1px);
    background-color: var(--bg-main);
  }

  .item-card.selected {
    border-color: var(--accent);
    background-color: color-mix(in srgb, var(--accent), transparent 92%);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .card-icon {
    color: var(--accent);
    opacity: 0.7;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .hash {
    font-weight: 800;
    font-size: 1.2rem;
  }

  .item-name {
    font-weight: 700;
    font-size: 1rem;
    color: var(--text-main);
    word-break: break-word;
  }

  .item-count {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-weight: 500;
  }

  .tag-item {
    border-color: color-mix(in srgb, var(--accent), transparent 80%);
  }
</style>
