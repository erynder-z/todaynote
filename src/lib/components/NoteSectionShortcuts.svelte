<script lang="ts">
  /**
   * Component for displaying note sections with shortcut hints for quick navigation.
   */

  import type { NoteSection } from '$lib/types/notes';
  import { tagSuggestionShortcuts } from '../config/shortcuts';
  import { inputManager } from '../stores/input.svelte';

  let { sections, onSelect } = $props<{
    sections: NoteSection[];
    onSelect: (name: string) => void;
  }>();

  // Show only up to 20 sections as there are only 20 shortcuts (1-9, A-K)
  let visibleSections = $derived(sections.slice(0, 20));
</script>

{#if visibleSections.length > 0}
  <div class="sections-container">
    {#each visibleSections as section, i}
      <button class="section-pill" onclick={() => onSelect(section.name)}>
        <span class="section-name">{section.name}</span>
        <span class="shortcut-hint">
          <span class="mod">{inputManager.primaryLabel}</span>
          <span class="mod">{inputManager.secondaryLabel}</span>
          <span class="key">{tagSuggestionShortcuts.labels[i]}</span>
        </span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .sections-container {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    margin-bottom: 1.5rem;
  }

  .section-pill {
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

  .section-pill:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 90%);
    color: var(--accent);
  }

  .section-name {
    font-weight: 500;
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

  .section-pill:hover .shortcut-hint {
    color: var(--text-main);
    /*     background-color: color-mix(in srgb, var(--bg-base), transparent 20%); */
    opacity: 1;
  }

  .section-pill:hover .mod {
    opacity: 1;
  }

  .mod {
    font-size: 0.6rem;
    opacity: 0.7;
  }

  .key {
    font-weight: 600;
  }
</style>
