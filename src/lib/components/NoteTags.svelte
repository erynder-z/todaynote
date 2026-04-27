<script lang="ts">
  /**
   * Component for displaying tags for a note.
   * Clicking the add button opens the Tag Manager modal.
   */
  import { removeNoteTag, sessionState, useShortcuts } from '$lib';
  import type { NoteContentResponse } from '$lib/types/notes';

  let { noteContent } = $props<{
    noteContent: NoteContentResponse | null;
  }>();

  let tags = $derived(noteContent?.metadata.tags || []);
  let selectedTag = $state<string | null>(null);

  /** Clears selection and opens the Tag Manager popup. */
  const openTagManager = () => {
    selectedTag = null;
    sessionState.activePopup = 'tagManager';
  };

  /** Toggles the selection of a tag pill. */
  const toggleSelectTag = (tag: string) => {
    if (selectedTag === tag) {
      selectedTag = null;
    } else {
      selectedTag = tag;
    }
  };

  /** Removes the currently selected tag from the note. */
  const handleRemoveTag = async () => {
    if (!selectedTag) return;
    const content = noteContent?.content || '';
    const updatedContent = await removeNoteTag(selectedTag, content);
    if (updatedContent) sessionState.todayNoteContent = updatedContent;
    selectedTag = null;
  };

  useShortcuts({
    manageTags: openTagManager,
  });
</script>

<div class="tags-container">
  {#each tags as tag}
    <button
      class="tag-pill"
      class:selected={selectedTag === tag}
      onclick={() => toggleSelectTag(tag)}
      title={tag}
    >
      {tag}
    </button>
  {/each}

  {#if selectedTag}
    <button
      class="add-tag-btn remove-mode"
      onclick={handleRemoveTag}
      aria-label="Remove selected tag"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1rem"
        viewBox="0 -960 960 960"
        width="1rem"
        fill="currentColor"
      >
        <path
          d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
        />
      </svg>
    </button>
  {:else}
    <button
      class="add-tag-btn"
      onclick={openTagManager}
      aria-label="Manage tags"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1rem"
        viewBox="0 -960 960 960"
        width="1rem"
        fill="currentColor"
      >
        <path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z" />
      </svg>
    </button>
  {/if}
</div>

<style>
  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
  }

  .tag-pill {
    font-size: 0.75rem;
    background-color: var(--accent);
    color: var(--accent-text);
    padding-inline: 0.625rem;
    padding-block: 0.125rem;
    border-radius: 999rem;
    font-weight: 600;
    white-space: nowrap;
    cursor: pointer;
    border: 0.125rem solid transparent;
    transition: all 0.2s ease;
    max-width: 12rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tag-pill.selected {
    background-color: var(--remove);
    color: var(--text-main);
    border-color: var(--remove);
  }

  .add-tag-btn {
    font-size: 0.75rem;
    background-color: var(--bg-surface);
    color: var(--accent);
    border: 0.0625rem dashed var(--border);
    padding-inline: 0.5rem;
    padding-block: 0.125rem;
    border-radius: 999rem;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 1.5rem;
    width: 2.25rem;
  }

  .add-tag-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .add-tag-btn.remove-mode {
    color: var(--remove);
    border: 0.0625rem solid var(--remove);
  }

  .add-tag-btn.remove-mode:hover {
    background-color: var(--remove);
    color: var(--text-main);
  }
</style>
