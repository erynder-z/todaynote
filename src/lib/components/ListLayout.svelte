<script lang="ts" generics="T">
  /**
   * Generic List Layout component.
   * Handles 1D navigation and selection via ListNavigator.
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
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="results-list" onmouseleave={() => nav.reset()}>
  {#each items as item, i}
    <button
      class="result-item"
      class:selected={i === nav.index}
      onclick={() => onSelect(item)}
      onmouseenter={() => {
        if (nav.shouldIgnoreMouseEnter()) return;
        nav.setIndex(i, 'mouse');
      }}
    >
      {@render itemSnippet(item, i)}
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
    padding: 0;
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
</style>
