<script lang="ts">
  /**
   * Displays list of all notes found in the user's notes folder.
   */
  import { ListNavigator, ModalFooter, sessionState, settings, t } from '$lib';
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

  const nav = new ListNavigator(
    () => notes.length,
    (i) => selectNote(notes[i]),
  );

  $effect(() => {
    if (settings.notesFolder) loadNotes();
  });
</script>

<!-- svelte-ignore a11y_autofocus -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="notes-container"
  onkeydown={(e) => nav.handleKey(e)}
  tabindex="-1"
  autofocus
>
  <main class="results-area" class:loading={isLoading}>
    {#if isLoading}
      <div class="status-view">
        <div class="spinner"></div>
        <p>{$t('notes.loading')}</p>
      </div>
    {:else if notes.length > 0}
      <div class="results-list" onmouseleave={() => nav.reset()}>
        {#each notes as note, i (note.filename)}
          <button
            class="result-item"
            class:selected={i === nav.index}
            onclick={() => selectNote(note)}
            onmouseenter={() => (nav.index = i)}
          >
            <div class="result-content">
              <span class="note-name">{note.formattedName}</span>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="status-view">
        <p class="muted">{$t('notes.list.empty')}</p>
      </div>
    {/if}
  </main>

  <ModalFooter
    shortcuts={[
      { label: $t('search.footer.navigate'), key: '↑↓' },
      { label: $t('search.footer.open'), key: 'Enter' },
      { label: $t('search.footer.close'), key: 'Esc' },
    ]}
    count={notes.length}
    countLabel={$t('notes.list.length', { count: notes.length })}
  />
</div>

<style>
  .notes-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background-color: var(--bg-main);
    overflow: hidden;
    border: 1px solid var(--border);
    outline: none;
  }

  .results-area {
    flex: 1;
    overflow-y: auto;
    min-height: 300px;
    position: relative;
  }

  .results-area.loading {
    opacity: 0.7;
  }

  .results-list {
    display: flex;
    flex-direction: column;
  }

  .result-item {
    display: flex;
    flex-direction: column;
    padding: 0.85rem 1rem;
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
  }

  .note-name {
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-main);
  }

  .result-item.selected .note-name {
    color: var(--accent);
  }

  .status-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1rem;
    color: var(--text-muted);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .muted {
    font-style: italic;
  }
</style>
