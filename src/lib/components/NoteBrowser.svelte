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
    useShortcuts,
  } from '$lib';
  import type { FormattedNote } from '$lib/types/notes';
  import { listNotes } from '$lib/utils/folder';
  import { readNoteContent } from '$lib/utils/notes';

  let notes: FormattedNote[] = $state([]);
  let totalCount = $state(0);
  let isLoading = $state(true);
  let isLoadingAll = $state(false);

  useShortcuts({
    toggleNoteBrowserLayout: () => {
      const nextLayout =
        settings.notesListLayout === 'list' ? 'masonry' : 'list';
      setLayout(nextLayout);
    },
  });

  /**
   * Fetches a list of notes with an optional limit from the backend.
   */
  const loadNotes = async (limit?: number) => {
    if (limit) isLoading = true;
    else isLoadingAll = true;

    const response = await listNotes(limit);
    if (response) {
      notes = response.notes;
      totalCount = response.totalCount;
    }

    isLoading = false;
    isLoadingAll = false;
  };

  /**
   * Fetches all available notes.
   */
  const loadAll = () => loadNotes();

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

  let masonryLayout: { handleKey: (e: KeyboardEvent) => boolean } | null =
    $state(null);

  const handleKeyDown = (e: KeyboardEvent) => {
    if (settings.notesListLayout === 'masonry' && masonryLayout)
      if (masonryLayout.handleKey(e)) return;
    nav.handleKey(e);
  };

  $effect(() => {
    if (settings.notesFolder) loadNotes(50);
  });

  $effect(() => {
    if (nav.index !== -1 && nav.lastInputSource === 'keyboard') {
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
<div class="notes-container" onkeydown={handleKeyDown} tabindex="-1" autofocus>
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
          height="1.25rem"
          viewBox="0 -960 960 960"
          width="1.25rem"
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
          height="1.25rem"
          viewBox="0 -960 960 960"
          width="1.25rem"
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
        <NotesMasonryLayout
          bind:this={masonryLayout}
          {notes}
          {nav}
          onSelect={selectNote}
        />
      {:else}
        <NotesListLayout {notes} {nav} onSelect={selectNote} />
      {/if}

      {#if notes.length < totalCount}
        <div class="load-more-container">
          <button
            class="load-more-btn"
            onclick={loadAll}
            disabled={isLoadingAll}
          >
            {#if isLoadingAll}
              <div class="spinner mini"></div>
              <span>{$t('notes.loading')}</span>
            {:else}
              <span>{$t('notes.list.load_all', { count: totalCount })}</span>
            {/if}
          </button>
        </div>
      {/if}
    {:else}
      <div class="status-view">
        <p class="muted">{$t('notes.list.empty')}</p>
      </div>
    {/if}
  </main>

  <ModalFooter
    shortcuts={[
      ...(settings.notesListLayout === 'masonry'
        ? [{ label: 'browser.navigate', key: '↑↓←→' }]
        : [{ label: $t('search.footer.navigate'), key: '↑↓' }]),
      { label: $t('search.footer.open'), key: 'Enter' },
      {
        label: $t('shortcuts.action.toggle_note_browser_layout'),
        action: 'toggleNoteBrowserLayout',
      },
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
    transition:
      background-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      color 0.15s cubic-bezier(0.2, 0, 0, 1);
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

  .spinner.mini {
    width: 16px;
    height: 16px;
    border-width: 2px;
  }

  @keyframes spin {
    to {
      transform: rotateZ(360deg);
    }
  }

  .load-more-container {
    display: flex;
    justify-content: center;
    padding: 2rem;
    background: linear-gradient(to bottom, transparent, var(--bg-main) 50%);
  }

  .load-more-btn {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 2rem;
    color: var(--text-main);
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .load-more-btn:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .load-more-btn:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .muted {
    font-style: italic;
  }
</style>
