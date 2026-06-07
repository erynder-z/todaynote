<script lang="ts">
  /**
   * Displays the note browser which allows switching between list and masonry layouts.
   */
  import type { FormattedNote } from '$lib/interfaces/notes';
  import { listNotes } from '$lib/utils/folder';
  import { formatNoteName, readNoteContent } from '$lib/utils/notes';
  import { ListNavigator } from '../stores/listNav.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import { settings } from '../stores/settings.svelte';
  import { toast } from '../stores/toast.svelte';
  import { locale, t } from '../utils/i18n';
  import { useShortcuts } from '../utils/shortcuts';
  import IdentIcon from './IdentIcon.svelte';
  import ListLayout from './ListLayout.svelte';
  import MasonryLayout from './MasonryLayout.svelte';
  import ModalFooter from './ModalFooter.svelte';

  let notes: FormattedNote[] = $state([]);
  let totalCount = $state(0);
  let isLoading = $state(true);
  let isLoadingAll = $state(false);

  useShortcuts({
    toggleNoteBrowserLayout: () => {
      const nextLayout = getNextLayout();
      settings.setNotesListLayout(nextLayout);
    },
  });

  /**
   * Returns the next layout to toggle to
   */
  const getNextLayout = () => {
    return settings.notesListLayout === 'list' ? 'masonry' : 'list';
  };

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

  /**
   * Returns the appropriate navigation label based on current layout
   */
  const getNavigationLabel = () => {
    return settings.notesListLayout === 'masonry'
      ? $t('browser.navigate')
      : $t('search.footer.navigate');
  };

  /**
   * Returns the appropriate navigation key based on current layout
   */
  const getNavigationKey = () => {
    return settings.notesListLayout === 'masonry' ? '↑↓←→' : '↑↓';
  };

  /**
   * Returns the appropriate word count translation based on note properties
   */
  const getWordCountText = (note: FormattedNote) => {
    if (note.hasCode) {
      return note.wordCount >= 2
        ? $t('notes.list.wordCountWithCode_multiple', { count: note.wordCount })
        : $t('notes.list.wordCountWithCode_single', { count: note.wordCount });
    } else {
      return note.wordCount >= 2
        ? $t('notes.list.wordCount_multiple', { count: note.wordCount })
        : $t('notes.list.wordCount_single', { count: note.wordCount });
    }
  };

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
</script>

{#snippet listSnippet(note: FormattedNote, i: number)}
  <div class="result-content">
    <span class="note-name">{formatNoteName(note.filename, $locale)}</span>
    {#if note.tags && note.tags.length > 0}
      <div class="list-tags">
        {#each note.tags as tag}
          <span class="tag-pill mini">{tag}</span>
        {/each}
      </div>
    {/if}
  </div>
{/snippet}

{#snippet masonrySnippet(note: FormattedNote, i: number)}
  <div class="card-header">
    <span class="note-name">{formatNoteName(note.filename, $locale)}</span>
  </div>

  {#if note.tags && note.tags.length > 0}
    <div class="note-tags">
      {#each note.tags as tag}
        <span class="tag-pill">{tag}</span>
      {/each}
    </div>
  {/if}

  {#if note.threads && note.threads.length > 0}
    <div class="note-threads">
      {#each note.threads as thread}
        <div class="thread-item">
          <IdentIcon title={thread} size={1} />
          <span>{thread}</span>
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
      <span>{getWordCountText(note)}</span>
    </div>
  </div>
{/snippet}

<!-- svelte-ignore a11y_autofocus -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="notes-container" onkeydown={handleKeyDown} tabindex="-1" autofocus>
  <main class="results-area" class:loading={isLoading}>
    {#if isLoading}
      <div class="status-view">
        <div class="spinner"></div>
        <p>{$t('notes.loading')}</p>
      </div>
    {:else if notes.length > 0}
      {#if settings.notesListLayout === 'masonry'}
        <MasonryLayout
          bind:this={masonryLayout}
          items={notes}
          {nav}
          onSelect={selectNote}
          itemSnippet={masonrySnippet}
        />
      {:else}
        <ListLayout
          items={notes}
          {nav}
          onSelect={selectNote}
          itemSnippet={listSnippet}
        />
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
      { label: getNavigationLabel(), key: getNavigationKey() },
      { label: $t('search.footer.open'), key: 'Enter' },
      {
        label: $t('shortcuts.action.toggle_note_browser_layout'),
        action: 'toggleNoteBrowserLayout',
      },
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

  .status-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    gap: 0.5rem;
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
    padding: 1rem;
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

  /* Snippet Styles */
  .result-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.85rem 1.5rem;
    width: 100%;
  }

  .note-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-main);
  }

  .list-tags {
    display: flex;
    gap: 0.3rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 1;
    min-width: 0;
  }

  .tag-pill.mini {
    font-size: 0.65rem;
    padding: 0.1rem 0.4rem;
  }

  .tag-pill {
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
    border-radius: 1rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .card-header {
    display: flex;
    align-items: center;
    margin-bottom: 0.75rem;
    padding: 0.75rem 1.25rem 0.25rem;
  }

  .note-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.5rem;
    padding: 0 1.25rem;
  }

  .note-threads {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 1rem;
    padding: 0 1.25rem;
  }

  .thread-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .thread-item svg {
    color: var(--accent);
    opacity: 0.7;
    flex-shrink: 0;
  }

  .note-footer {
    margin-top: auto;
    padding: 0.5rem 1.25rem 1.25rem;
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
