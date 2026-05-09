<script lang="ts">
  /**
   * Displays the note browser which allows switching between list and masonry layouts.
   */
  import {
    ListNavigator,
    ModalFooter,
    NotesListLayout,
    NotesMasonryLayout,
    sessionState,
    settings,
    t,
    toast,
  } from '$lib';
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
    } else {
      toast.error($t('notes.error.load'));
    }
  };

  const nav = new ListNavigator(
    () => notes.length,
    (i) => selectNote(notes[i]),
  );

  $effect(() => {
    if (settings.notesFolder) loadNotes();
  });

  $effect(() => {
    if (nav.index !== -1) {
      const selected = document.querySelector(
        '.note-card.selected, .result-item.selected',
      );
      selected?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  });
  const setLayout = (layout: 'list' | 'masonry') => {
    settings.save({
      notesFolder: settings.notesFolder,
      locale: settings.locale,
      theme: settings.theme,
      rememberWindowSize: settings.rememberWindowSize,
      notesListLayout: layout,
    });
  };
</script>

<!-- svelte-ignore a11y_autofocus -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="notes-container"
  onkeydown={(e) => nav.handleKey(e)}
  tabindex="-1"
  autofocus
>
  <div class="layout-toolbar">
    <div class="toggle-group">
      <button
        class="toggle-btn"
        class:active={settings.notesListLayout === 'list'}
        onclick={() => setLayout('list')}
        title="List View"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="20px"
          viewBox="0 -960 960 960"
          width="20px"
          fill="currentColor"
          ><path
            d="M240-240v-120h480v120H240Zm0-200v-120h480v120H240Zm0-200v-120h480v120H240Z"
          /></svg
        >
      </button>
      <button
        class="toggle-btn"
        class:active={settings.notesListLayout === 'masonry'}
        onclick={() => setLayout('masonry')}
        title="Masonry View"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="20px"
          viewBox="0 -960 960 960"
          width="20px"
          fill="currentColor"
          ><path
            d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h560q33 0 56.5 23.5T840-760v560q0-33-23.5-56.5T760-120H200Zm0-80h240v-180H200v180Zm320 0h240v-300H520v300ZM200-460h240v-300H200v300Zm320 0h240v-180H520v180Z"
          /></svg
        >
      </button>
    </div>
  </div>

  <main class="results-area" class:loading={isLoading}>
    {#if isLoading}
      <div class="status-view">
        <div class="spinner"></div>
        <p>{$t('notes.loading')}</p>
      </div>
    {:else if notes.length > 0}
      {#if settings.notesListLayout === 'masonry'}
        <NotesMasonryLayout {notes} {nav} onSelect={selectNote} />
      {:else}
        <NotesListLayout {notes} {nav} onSelect={selectNote} />
      {/if}
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

  .layout-toolbar {
    display: flex;
    justify-content: flex-end;
    padding: 0.75rem 1.5rem;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
  }

  .toggle-group {
    display: flex;
    background-color: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 2px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.4rem;
    background: none;
    border: none;
    border-radius: 0.35rem;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.2s;
  }

  .toggle-btn:hover {
    color: var(--text-main);
  }

  .toggle-btn.active {
    background-color: var(--accent);
    color: white;
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
