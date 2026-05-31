<script lang="ts" generics="T">
  /**
   * Generic Masonry Layout component.
   * Handles responsive columns, distribution, and 2D keyboard navigation.
   */
  import type { Snippet } from 'svelte';
  import type { ListNavigator } from '$lib/stores/listNav.svelte';

  let {
    items,
    nav,
    onSelect,
    itemSnippet,
  }: {
    items: T[];
    nav: ListNavigator;
    onSelect: (item: T) => void;
    itemSnippet: Snippet<[T, number]>;
  } = $props();

  let containerWidth = $state(0);

  /**
   * Number of columns based on container width.
   */
  let columnCount = $derived.by(() => {
    if (containerWidth > 1200) return 5;
    if (containerWidth > 1024) return 4;
    if (containerWidth > 768) return 3;
    if (containerWidth > 500) return 2;
    return 1;
  });

  /**
   * Distributes items into columns for masonry layout.
   */
  let columns = $derived.by(() => {
    const cols: T[][] = Array.from({ length: columnCount }, () => []);

    for (let i = 0; i < items.length; i++) {
      cols[i % columnCount].push(items[i]);
    }

    return cols;
  });

  /**
   * Handles grid navigation with arrow keys.
   */
  export const handleKey = (e: KeyboardEvent) => {
    if (['ArrowRight', 'ArrowLeft', 'ArrowUp', 'ArrowDown'].includes(e.key)) {
      e.preventDefault();

      const currentIndex = nav.index === -1 ? 0 : nav.index;
      let nextIndex = currentIndex;

      if (e.key === 'ArrowLeft') {
        nextIndex = Math.max(0, currentIndex - 1);
      } else if (e.key === 'ArrowRight') {
        nextIndex = Math.min(items.length - 1, currentIndex + 1);
      } else if (e.key === 'ArrowUp') {
        nextIndex = Math.max(0, currentIndex - columnCount);
      } else if (e.key === 'ArrowDown') {
        nextIndex = Math.min(items.length - 1, currentIndex + columnCount);
      }

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
  <div class="columns-wrapper" style="--cols: {columnCount}">
    {#each columns as column, c}
      <div class="column-lane">
        {#each column as item, r}
          {@const globalIdx = r * columnCount + c}
          {#if globalIdx < items.length}
            <button
              class="item-card"
              class:selected={globalIdx === nav.index}
              onclick={() => onSelect(item)}
              onmouseenter={() => {
                if (nav.shouldIgnoreMouseEnter()) return;
                nav.setIndex(globalIdx, 'mouse');
              }}
            >
              {@render itemSnippet(item, globalIdx)}
            </button>
          {/if}
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
    gap: 1.5rem;
    align-items: flex-start;
  }

  .column-lane {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    min-width: 0;
  }

  .item-card {
    display: flex;
    flex-direction: column;
    padding: 0;
    background: none;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    text-align: left;
    cursor: pointer;
    width: 100%;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
    overflow: hidden;
  }

  .item-card.selected {
    border-color: var(--accent);
    background-color: color-mix(in srgb, var(--accent), transparent 92%);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
</style>
