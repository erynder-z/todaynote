<script lang="ts">
  /**
   * Displays list of all notes found in the user's notes folder.
   */
  import { sessionState, settings, t } from '$lib';
  import type { FormattedNote } from '$lib/types/notes';
  import { listNotes } from '$lib/utils/folder';
  import { readNoteContent } from '$lib/utils/notes';

  let notes: FormattedNote[] = $state([]);
  let isLoading = $state(true);

  /**
   * Fetches the list of all available notes from the backend.
   */
  const loadNotes = async () => {
    isLoading = true;
    const loadedNotes = await listNotes();
    if (loadedNotes) notes = loadedNotes;
    isLoading = false;
  };

  /**
   * Loads the content of a specific note and sets it as active in the app.
   */
  const selectNote = async (note: FormattedNote) => {
    if (!settings.notesFolder) return;
    const path = `${settings.notesFolder}/${note.filename}`;
    const content = await readNoteContent(path);
    if (content !== null) {
      sessionState.todayNotePath = path;
      sessionState.todayNoteContent = content;
      sessionState.activePopup = null;
    }
  };

  $effect(() => {
    if (settings.notesFolder) loadNotes();
  });
</script>

<div class="notes-section">
  {#if isLoading}
    <div class="status-msg">{$t('notes.loading')}</div>
  {:else if notes.length > 0}
    <div class="notes-list">
      {#each notes as note (note.filename)}
        <div
          role="button"
          tabindex="0"
          class="note-item"
          onclick={() => selectNote(note)}
          onkeydown={(e) => e.key === 'Enter' && selectNote(note)}
        >
          <span class="note-name">{note.formattedName}</span>
        </div>
      {/each}
    </div>
  {:else}
    <div class="status-msg">{$t('notes.list.empty')}</div>
  {/if}
</div>

<style>
  .notes-section {
    margin: 1.5rem 0;
  }

  .notes-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-width: 60ch;
    margin: 0 auto;
  }

  .note-item {
    padding: 1rem 1.25rem;
    background-color: var(--bg-surface);
    border-radius: 0.5rem;
    border: 0.0625rem solid var(--border);
    cursor: pointer;
    transition:
      transform 0.15s,
      border-color 0.15s,
      background-color 0.15s;
    color: var(--text-main);
    display: flex;
    align-items: center;
  }

  .note-item:hover {
    transform: translateX(0.5ch);
    border-color: var(--accent);
    background-color: var(--bg-base);
  }

  .note-name {
    font-size: 1rem;
    font-weight: 500;
  }

  .status-msg {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
    font-style: italic;
  }
</style>
