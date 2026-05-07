<script lang="ts">
  /**
   * Displays list of all notes found in the user's notes folder.
   */
  import {
    ListNavigator,
    ModalFooter,
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
      const selected = document.querySelector('.note-card.selected');
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
      <div
        class={settings.notesListLayout === 'masonry'
          ? 'notes-grid'
          : 'results-list'}
        onmouseleave={() => nav.reset()}
      >
        {#each notes as note, i (note.filename)}
          <button
            class={settings.notesListLayout === 'masonry'
              ? 'note-card'
              : 'result-item'}
            class:selected={i === nav.index}
            onclick={() => selectNote(note)}
            onmouseenter={() => (nav.index = i)}
          >
            {#if settings.notesListLayout === 'masonry'}
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

              {#if note.preview}
                <p class="note-preview">{note.preview}</p>
              {/if}
            {:else}
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
            {/if}
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

  .list-tags {
    display: flex;
    gap: 0.3rem;
  }

  .tag-pill.mini {
    font-size: 0.65rem;
    padding: 0.1rem 0.4rem;
  }

  .notes-grid {
    column-count: 3;
    column-gap: 1rem;
    padding: 1.5rem;
    width: 100%;
  }

  @media (max-width: 1200px) {
    .notes-grid {
      column-count: 2;
    }
  }

  @media (max-width: 768px) {
    .notes-grid {
      column-count: 1;
    }
  }

  .note-card {
    break-inside: avoid;
    display: flex;
    flex-direction: column;
    padding: 1.25rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.75rem;
    text-align: left;
    cursor: pointer;
    width: 100%;
    margin-bottom: 1rem;
    transition: all 0.2s ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }

  .note-card.selected {
    border-color: var(--accent);
    background-color: color-mix(in srgb, var(--accent), transparent 92%);
    transform: translateY(-2px);
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

  .note-preview {
    font-size: 0.875rem;
    color: var(--text-muted);
    line-height: 1.5;
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 4;
    -webkit-box-orient: vertical;
    overflow: hidden;
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
