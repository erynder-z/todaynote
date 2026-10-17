<script lang="ts">
  /**
   * Displays all pinned threads in a list or masonry layout.
   */
  import type { PinnedThreadItem } from '#lib/interfaces/notes.ts';
  import { notesService } from '#lib/utils/notes.ts';
  import { ListNavigator } from '../stores/listNav.svelte.ts';
  import { sessionState } from '../stores/sessionState.svelte.ts';
  import { settings } from '../stores/settings.svelte.ts';
  import { toast } from '../stores/toast.svelte.ts';
  import { t } from '../utils/i18n.ts';
  import { useShortcuts } from '../utils/shortcuts.ts';
  import ListLayout from './ListLayout.svelte';
  import MasonryLayout from './MasonryLayout.svelte';
  import ModalFooter from './ModalFooter.svelte';

  let pinnedThreads: PinnedThreadItem[] = $state([]);
  let isLoading = $state(true);

  useShortcuts({
    toggleNoteBrowserLayout: () => {
      const nextLayout =
        settings.notesListLayout === 'list' ? 'masonry' : 'list';
      settings.setNotesListLayout(nextLayout);
    },
  });

  let masonryLayout: { handleKey: (e: KeyboardEvent) => boolean } | null =
    $state(null);

  /**
   * Loads all pinned threads.
   */
  const loadPinnedThreads = async () => {
    isLoading = true;
    try {
      pinnedThreads = await notesService.getPinnedThreads();
    } catch (error) {
      console.error('Error loading pinned threads:', error);
      toast.error($t('notes.error.load'));
    } finally {
      isLoading = false;
    }
  };

  /**
   * Loads and displays the selected pinned thread in a single thread view.
   */
  const selectPinnedThread = async (thread: PinnedThreadItem) => {
    if (!settings.notesFolder) return;

    // Get the thread content
    const threadContent = await notesService.getThreadContent(
      thread.filename,
      thread.threadId,
    );

    if (threadContent !== null) {
      sessionState.threadViewContent = threadContent;
      sessionState.threadViewName = thread.threadName;
      sessionState.threadViewThreadId = thread.threadId;
      sessionState.threadViewFilename = thread.filename;
      sessionState.activePopup = 'threadView';
    } else {
      toast.error($t('notes.error.load'));
    }
  };

  const nav = new ListNavigator(
    () => pinnedThreads.length,
    (i) => selectPinnedThread(pinnedThreads[i]),
  );

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
    return settings.notesListLayout === 'masonry' ? '↑ ↓ ← →' : '↑ ↓';
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (pinnedThreads.length === 0) return;

    if (settings.notesListLayout === 'masonry' && masonryLayout)
      if (masonryLayout.handleKey(e)) return;
    nav.handleKey(e);
  };

  $effect(() => {
    if (settings.notesFolder) loadPinnedThreads();
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

{#snippet listSnippet(thread: PinnedThreadItem, i: number)}
  <div class="result-content">
    <div class="thread-header">
      <span class="thread-name">{thread.threadName}</span>
      <span class="thread-source">{thread.formattedDate}</span>
    </div>
    <div class="thread-excerpt">{thread.excerpt}</div>
  </div>
{/snippet}

{#snippet masonrySnippet(thread: PinnedThreadItem, i: number)}
  <div class="card-content">
    <div class="card-header">
      <span class="thread-name">{thread.threadName}</span>
    </div>
    <div class="thread-source">{thread.formattedDate}</div>
    <div class="thread-excerpt">{thread.excerpt}</div>
  </div>
{/snippet}

<!-- svelte-ignore a11y_autofocus -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="pinned-threads-container"
  onkeydown={handleKeyDown}
  tabindex="-1"
  autofocus
>
  <main class="results-area" class:loading={isLoading}>
    {#if isLoading}
      <div class="status-view">
        <div class="spinner"></div>
        <p>{$t('notes.loading')}</p>
      </div>
    {:else if pinnedThreads.length > 0}
      {#if settings.notesListLayout === 'masonry'}
        <MasonryLayout
          bind:this={masonryLayout}
          items={pinnedThreads}
          {nav}
          onSelect={selectPinnedThread}
          itemSnippet={masonrySnippet}
        />
      {:else}
        <ListLayout
          items={pinnedThreads}
          {nav}
          onSelect={selectPinnedThread}
          itemSnippet={listSnippet}
        />
      {/if}
    {:else}
      <div class="status-view">
        <p class="muted">{$t('pinned_threads.empty')}</p>
      </div>
    {/if}
  </main>

  <ModalFooter
    shortcuts={[
      { label: getNavigationLabel(), key: getNavigationKey() },
      { label: $t('search.footer.open'), key: '↵' },
      {
        label: $t('shortcuts.action.toggle_note_browser_layout'),
        action: 'toggleNoteBrowserLayout',
      },
    ]}
    count={pinnedThreads.length}
    countLabel={$t('pinned_threads.count', { count: pinnedThreads.length })}
  />
</div>

<style>
  .pinned-threads-container {
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

  @keyframes spin {
    to {
      transform: rotateZ(360deg);
    }
  }

  .muted {
    font-style: italic;
  }

  /* List Layout Styles */
  .result-content {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.85rem 1.5rem;
    width: 100%;
  }

  .thread-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .thread-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--thread-marker-color);
  }

  .thread-source {
    font-size: 0.85rem;
    color: var(--text-muted);
    font-style: italic;
  }

  .thread-excerpt {
    font-size: 0.9rem;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Masonry Layout Styles */
  .card-content {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1.25rem;
    width: 100%;
  }

  .card-header {
    display: flex;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .card-content .thread-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--thread-marker-color);
  }

  .card-content .thread-source {
    font-size: 0.8rem;
    color: var(--text-muted);
    font-style: italic;
  }

  .card-content .thread-excerpt {
    font-size: 0.9rem;
    color: var(--text-muted);
    display: -webkit-box;
    display: box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    box-orient: vertical;
    overflow: hidden;
    line-height: 1.4;
  }
</style>
