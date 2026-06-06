<script lang="ts">
  /**
   * Component for displaying note threads with shortcut hints for quick navigation.
   */
  import type { NoteThread } from '$lib/interfaces/notes';
  import { tagSuggestionShortcuts } from '../config/shortcuts';
  import IdentIcon from './IdentIcon.svelte';
  import KeyboardShortcut from './KeyboardShortcut.svelte';

  let { threads, onSelect } = $props<{
    threads: NoteThread[];
    onSelect: (name: string) => void;
  }>();

  // Show only up to 20 threads as there are only 20 shortcuts (1-9, A-K)
  let visibleThreads = $derived(threads.slice(0, 20));
</script>

{#if visibleThreads.length > 0}
  <div class="threads-container">
    {#each visibleThreads as thread, i}
      <button class="thread-pill" onclick={() => onSelect(thread.name)}>
        <div class="left-side">
          <IdentIcon title={thread.name} size={16} />
          <span class="thread-name">{thread.name}</span>
        </div>
        <div class="right-side">
          <span class="shortcut-hint">
            <KeyboardShortcut
              primary
              secondary
              key={tagSuggestionShortcuts.labels[i]}
            />
          </span>
        </div>
      </button>
    {/each}
  </div>
{/if}

<style>
  .threads-container {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    margin-bottom: 1.5rem;
  }

  .thread-pill {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.4rem 0.6rem;
    background: transparent;
    border: none;
    border-bottom: dashed 1px var(--border);
    color: var(--text-main);
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.1s ease;
    text-align: left;
    width: 100%;
  }

  .thread-pill:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 90%);
  }

  .thread-pill .left-side {
    display: flex;
    gap: 0.5rem;
    font-weight: 500;
    overflow: hidden;
    white-space: nowrap;
    min-width: 0;
  }

  .thread-pill .thread-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .shortcut-hint {
    font-family: var(--font-mono, monospace);
    font-size: 0.65rem;
    color: var(--text-muted);
    background-color: color-mix(in srgb, var(--bg-base), transparent 50%);
    padding: 0.1rem 0.2rem;
    border-radius: 0.2rem;
    display: flex;
    align-items: center;
    gap: 0.1rem;
    opacity: 0.6;
    flex-shrink: 0;
  }
</style>
