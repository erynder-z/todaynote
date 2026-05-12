<script lang="ts">
  import { t } from '$lib';
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

  let containerWidth = $state(0);
  let columnCount = $derived.by(() => {
    if (containerWidth > 1200) return 4;
    if (containerWidth > 768) return 3;
    return 1;
  });

  let columns = $derived.by(() => {
    const cols: FormattedNote[][] = Array.from(
      { length: columnCount },
      () => [],
    );

    const base = Math.floor(notes.length / columnCount);
    const remainder = notes.length % columnCount;

    let currentIdx = 0;
    for (let c = 0; c < columnCount; c++) {
      const size = base + (c < remainder ? 1 : 0);
      cols[c] = notes.slice(currentIdx, currentIdx + size);
      currentIdx += size;
    }

    return cols;
  });

  const getPositionFromIndex = (index: number) => {
    let accumulated = 0;
    for (let c = 0; c < columns.length; c++) {
      if (index < accumulated + columns[c].length) {
        return { col: c, row: index - accumulated };
      }
      accumulated += columns[c].length;
    }
    return { col: 0, row: 0 };
  };

  const getIndexFromPosition = (col: number, row: number) => {
    let accumulated = 0;
    for (let c = 0; c < col; c++) {
      accumulated += columns[c].length;
    }
    return accumulated + Math.min(row, columns[col].length - 1);
  };

  /**
   * Handles grid navigation with arrow keys
   */
  export const handleKey = (e: KeyboardEvent) => {
    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
      e.preventDefault();

      const pos = getPositionFromIndex(nav.index === -1 ? 0 : nav.index);
      let nextCol = pos.col;

      if (e.key === 'ArrowLeft') {
        nextCol = (pos.col - 1 + columnCount) % columnCount;
      } else {
        nextCol = (pos.col + 1) % columnCount;
      }

      // Preserve relative vertical position if possible
      const nextIndex = getIndexFromPosition(nextCol, pos.row);
      nav.setIndex(nextIndex, 'keyboard');
      return true;
    }

    // Default Up/Down handling via the navigator
    return nav.handleKey(e);
  };
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="notes-masonry-container"
  bind:clientWidth={containerWidth}
  onmouseleave={() => nav.reset()}
>
  <div class="columns-wrapper" style="--cols: {columnCount}">
    {#each columns as column}
      <div class="column-lane">
        {#each column as note}
          {@const globalIdx = notes.indexOf(note)}
          <button
            class="note-card"
            class:selected={globalIdx === nav.index}
            onclick={() => onSelect(note)}
            onmouseenter={() => {
              if (nav.shouldIgnoreMouseEnter()) return;
              nav.setIndex(globalIdx, 'mouse');
            }}
          >
            <div class="card-header">
              <span class="note-name">{note.formattedName}</span>
            </div>

            {#if note.tags && note.tags.length > 0}
              <div class="note-tags">
                {#each note.tags as tag}
                  <span class="tag-pill">{tag}</span>
                {/each}
              </div>
            {/if}

            {#if note.sections && note.sections.length > 0}
              <div class="note-sections">
                {#each note.sections as section}
                  <div class="section-item">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      height="1rem"
                      viewBox="0 -960 960 960"
                      width="1rem"
                      fill="currentColor"
                      ><path
                        d="m382-354 182-182-182-182 56-56 238 238-238 238-56-56Z"
                      /></svg
                    >
                    <span>{section}</span>
                  </div>
                {/each}
              </div>
            {/if}

            <div class="note-footer">
              <div class="note-stats">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  height="1rem"
                  viewBox="0 -960 960 960"
                  width="1rem"
                  fill="currentColor"
                  ><path
                    d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h560q33 0 56.5 23.5T840-760v560q0-33-23.5-56.5T760-120H200Zm0-80h560v-560H200v560Zm80-80h400v-80H280v80Zm0-160h400v-80H280v80Zm0-160h400v-80H280v80Z"
                  /></svg
                >
                <span
                  >{$t('notes.list.wordCount', {
                    count: note.wordCount,
                  })}</span
                >
              </div>
            </div>
          </button>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .notes-masonry-container {
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
    gap: 1rem;
    min-width: 0;
  }

  .note-card {
    display: flex;
    flex-direction: column;
    padding: 1.25rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    text-align: left;
    cursor: pointer;
    width: 100%;
    transition:
      background-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      border-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      box-shadow 0.15s cubic-bezier(0.2, 0, 0, 1);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }

  .note-card.selected {
    border-color: var(--accent);
    background-color: color-mix(in srgb, var(--accent), transparent 92%);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .card-header {
    display: flex;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .note-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-main);
  }

  .selected .note-name {
    color: var(--accent);
  }

  .note-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.75rem;
  }

  .tag-pill {
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
    border-radius: 1rem;
    font-weight: 500;
  }

  .note-sections {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 1rem;
  }

  .section-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .section-item svg {
    color: var(--accent);
    opacity: 0.7;
    flex-shrink: 0;
  }

  .note-footer {
    margin-top: auto;
    padding-top: 0.75rem;
    border-top: 1px solid color-mix(in srgb, var(--border), transparent 50%);
  }

  .note-stats {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .note-stats svg {
    opacity: 0.6;
  }
</style>
