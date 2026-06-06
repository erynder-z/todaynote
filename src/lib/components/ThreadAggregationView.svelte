<script lang="ts">
  /**
   * Component for displaying aggregated content (threads) from multiple notes.
   * Shows each content block with a date hint that links back to the source note.
   */
  import { sessionState, settings, t, toast } from '$lib';
  import { locale } from '$lib/utils/i18n';
  import { formatNoteName, readNoteContent } from '$lib/utils/notes';
  import IdentIcon from './IdentIcon.svelte';
  import MilkdownEditor from './MilkdownEditor.svelte';

  let aggregation = $derived(sessionState.aggregatedThread);

  /**
   * Opens the original note when the date link button is clicked
   */
  const openOriginalNote = async (filename: string) => {
    if (!settings.notesFolder) return;
    const path = `${settings.notesFolder}/${filename}`;
    const content = await readNoteContent(path);
    if (content !== null) {
      sessionState.todayNotePath = path;
      sessionState.todayNoteContent = content;
      sessionState.activePopup = null;
    } else {
      toast.error($t('notes.error.load'));
    }
  };
</script>

<div class="aggregation-container">
  {#if aggregation && aggregation.items.length > 0}
    <div class="items-list">
      {#each aggregation.items as item}
        <div class="aggregation-item">
          <header class="item-header">
            <IdentIcon title={aggregation.threadName} size={24} />
            <button
              class="date-link"
              onclick={() => openOriginalNote(item.filename)}
              title="Open original note"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                height="1rem"
                viewBox="0 -960 960 960"
                width="1rem"
                fill="currentColor"
                ><path
                  d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h560v-280h80v280q0 33-23.5 56.5T784-120H200Zm392-520-56-56 224-224H600v-80h280v280h-80v-168L592-640Z"
                /></svg
              >
              {formatNoteName(item.filename, $locale)}
            </button>
          </header>
          <div class="item-body">
            <MilkdownEditor content={item.content} readonly />
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <p class="muted">No content found for this thread.</p>
    </div>
  {/if}
</div>

<style>
  .aggregation-container {
    padding: 1rem;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .items-list {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    flex: 1;
    overflow-y: auto;
    padding-right: 0.5rem;
  }

  .aggregation-item {
    border-bottom: 1px solid var(--border);
    padding-bottom: 1rem;
  }

  .aggregation-item:last-child {
    border-bottom: none;
  }

  .item-header {
    margin-bottom: 0.5rem;
    display: flex;
    justify-content: flex-start;
    align-items: center;
    gap: 0.75rem;
  }

  .date-link {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: color-mix(in srgb, var(--accent), transparent 90%);
    border: 1px solid color-mix(in srgb, var(--accent), transparent 80%);
    color: var(--accent);
    padding: 0.4rem 0.8rem;
    border-radius: 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .date-link:hover {
    background: var(--accent);
    color: var(--accent-text);
    border-color: var(--accent);
  }

  .item-body {
    background-color: var(--bg-surface);
    border-radius: 0.5rem;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border);
    margin-top: 0.5rem;
  }

  /* Override Milkdown min-height for these small blocks */
  .item-body :global(.milkdown) {
    min-height: auto !important;
  }

  .empty-state {
    display: flex;
    justify-content: center;
    align-items: center;
    flex: 1;
    color: var(--text-muted);
    font-style: italic;
  }

  .muted {
    opacity: 0.7;
  }
</style>
