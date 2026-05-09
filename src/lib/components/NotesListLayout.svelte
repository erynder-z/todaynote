<script lang="ts">
  import type { ListNavigator } from '$lib/stores/listNav.svelte';
  import type { FormattedNote } from '$lib/types/notes';

  let {
    notes,
    nav,
    onSelect,
  }: {
    notes: FormattedNote[];
    nav: ListNavigator;
    onSelect: (note: FormattedNote) => void;
  } = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="results-list" onmouseleave={() => nav.reset()}>
  {#each notes as note, i (note.filename)}
    <button
      class="result-item"
      class:selected={i === nav.index}
      onclick={() => onSelect(note)}
      onmouseenter={() => (nav.index = i)}
    >
      <div class="result-content">
        <span class="note-name">{note.formattedName}</span>
        {#if note.tags && note.tags.length > 0}
          <div class="list-tags">
            {#each note.tags as tag}
              <span class="tag-pill mini">{tag}</span>
            {/each}
          </div>
        {/if}
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
    padding: 0.85rem 1.5rem;
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

  .result-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .note-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-main);
  }

  .selected .note-name {
    color: var(--accent);
  }

  .list-tags {
    display: flex;
    gap: 0.3rem;
  }

  .tag-pill.mini {
    font-size: 0.65rem;
    padding: 0.1rem 0.4rem;
  }

  .tag-pill {
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
    border-radius: 1rem;
    font-weight: 500;
  }
</style>
