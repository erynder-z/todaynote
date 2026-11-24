<script lang="ts">
  /**
   * Orchestrates which modal is currently displayed based on
   * the `sessionState.activePopup` state.
   */
  import { flip } from 'svelte/animate';
  import { fade } from 'svelte/transition';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import type { LazyPopup, LoadedPopupComponents } from '#lib/types/ui.ts';
  import { toggleFindInNote } from '#lib/utils/findInElement.ts';
  import { sessionState } from '../stores/sessionState.svelte.ts';
  import { settings } from '../stores/settings.svelte.ts';
  import { t } from '../utils/i18n.ts';
  import { useShortcuts } from '../utils/shortcuts.ts';
  import FindInView from './FindInNote.svelte';
  import Modal from './Modal.svelte';
  import ThreadOptionsMenu from './ThreadOptionsMenu.svelte';
  import Toast from './Toast.svelte';

  useShortcuts({
    toggleFindInNote,
  });

  // Popup views are loaded on demand.
  // `satisfies` enforces that every popup has a loader.
  const popupLoaders = {
    settings: () => import('./SettingsView.svelte'),
    noteBrowser: () => import('./NoteBrowser.svelte'),
    noteMetadata: () => import('./NoteMetadataView.svelte'),
    pinnedThreads: () => import('./PinnedThreads.svelte'),
    search: () => import('./SearchNotes.svelte'),
    tagManager: () => import('./TagManagerModal.svelte'),
    shortcuts: () => import('./ShortcutListModal.svelte'),
    threadAggregation: () => import('./ThreadAggregationView.svelte'),
    threadView: () => import('./ThreadView.svelte'),
    about: () => import('./AboutView.svelte'),
    statistics: () => import('./StatisticsView.svelte'),
    addImage: () => import('./AddImageURLModal.svelte'),
  } satisfies Record<LazyPopup, () => Promise<unknown>>;

  type LoadedPopups = LoadedPopupComponents<typeof popupLoaders>;

  const loadedPopups = $state<LoadedPopups>({});

  function loadPopup<K extends LazyPopup>(popup: K) {
    popupLoaders[popup]().then((mod) => {
      loadedPopups[popup] = mod.default as LoadedPopups[K];
    });
  }

  $effect(() => {
    const popup = sessionState.activePopup;
    if (popup && popup !== 'threadOptions' && !loadedPopups[popup]) {
      loadPopup(popup);
    }
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
{#if sessionState.activePopup && sessionState.activePopup !== 'threadOptions'}
  <div
    transition:fade={{ duration: sessionState.isLinux ? 0 : 150 }}
    class="overlay"
    onclick={() => (sessionState.activePopup = null)}
  >
    {#if sessionState.activePopup === 'settings' && loadedPopups.settings}
      {@const PopupView = loadedPopups.settings}
      <Modal title={$t('settings.title')} size="xl">
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'noteBrowser' && loadedPopups.noteBrowser}
      {@const PopupView = loadedPopups.noteBrowser}
      <Modal
        title={$t('notes.list.title')}
        size={settings.notesListLayout === 'masonry' ? 'xl' : 'md'}
        showLayoutToggle={true}
      >
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'noteMetadata' && loadedPopups.noteMetadata}
      {@const PopupView = loadedPopups.noteMetadata}
      <Modal title={$t('note.metadata.title')} size="md">
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'pinnedThreads' && loadedPopups.pinnedThreads}
      {@const PopupView = loadedPopups.pinnedThreads}
      <Modal
        title={$t('pinned_threads.title')}
        size={settings.notesListLayout === 'masonry' ? 'xl' : 'md'}
        showLayoutToggle={true}
      >
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'search' && loadedPopups.search}
      {@const PopupView = loadedPopups.search}
      <Modal
        title={$t('search.title')}
        size={settings.notesListLayout === 'masonry' ? 'xl' : 'lg'}
        showLayoutToggle={true}
      >
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'tagManager' && loadedPopups.tagManager}
      {@const PopupView = loadedPopups.tagManager}
      <Modal
        title={$t('tag.manager.title')}
        size={settings.notesListLayout === 'masonry' ? 'xl' : 'md'}
        showLayoutToggle={true}
      >
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'shortcuts' && loadedPopups.shortcuts}
      {@const PopupView = loadedPopups.shortcuts}
      <Modal title={$t('shortcuts.title')} size="lg">
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'threadAggregation' && loadedPopups.threadAggregation}
      {@const PopupView = loadedPopups.threadAggregation}
      <Modal title={sessionState.aggregatedThread?.threadName} size="xl">
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'threadView' && loadedPopups.threadView}
      {@const PopupView = loadedPopups.threadView}
      <Modal title={sessionState.threadViewName || 'Thread'} size="lg">
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'about' && loadedPopups.about}
      {@const PopupView = loadedPopups.about}
      <Modal title="About TodayNote" size="md">
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'statistics' && loadedPopups.statistics}
      {@const PopupView = loadedPopups.statistics}
      <Modal title={$t('statistics.title')} size="lg">
        <PopupView />
      </Modal>
    {:else if sessionState.activePopup === 'addImage' && loadedPopups.addImage}
      {@const PopupView = loadedPopups.addImage}
      <Modal title={$t('image.title')} size="minimal">
        <PopupView />
      </Modal>
    {/if}
  </div>
{/if}

{#if sessionState.activePopup === 'threadOptions' && sessionState.selectedThreadForOptions}
  <ThreadOptionsMenu thread={sessionState.selectedThreadForOptions} />
{/if}

<div class="toast-container">
  {#each toast.toasts as item (item.id)}
    <div animate:flip={{ duration: sessionState.isLinux ? 0 : 200 }}>
      <Toast t={item} />
    </div>
  {/each}
</div>

<FindInView />

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    z-index: 2000;
    backdrop-filter: brightness(60%);
  }

  :global(html.linux) .overlay {
    backdrop-filter: none;
    background-color: rgb(0 0 0 / 0.5);
  }

  .toast-container {
    position: fixed;
    bottom: 1.5rem;
    right: 1.5rem;
    z-index: 5000;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    pointer-events: none;
    gap: 0.5rem;
  }
</style>
