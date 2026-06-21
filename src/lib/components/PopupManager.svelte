<script lang="ts">
  /**
   * Orchestrates which modal is currently displayed based on
   * the `sessionState.activePopup` state.
   */
  import { flip } from 'svelte/animate';
  import { fade } from 'svelte/transition';
  import { toast } from '$lib/stores/toast.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import { settings } from '../stores/settings.svelte';
  import { t } from '../utils/i18n';
  import AboutView from './AboutView.svelte';
  import Modal from './Modal.svelte';
  import NoteBrowser from './NoteBrowser.svelte';
  import SearchNotes from './SearchNotes.svelte';
  import SettingsView from './SettingsView.svelte';
  import ShortcutListModal from './ShortcutListModal.svelte';
  import StatisticsView from './StatisticsView.svelte';
  import TagManagerModal from './TagManagerModal.svelte';
  import ThreadAggregationView from './ThreadAggregationView.svelte';
  import ThreadOptionsMenu from './ThreadOptionsMenu.svelte';
  import Toast from './Toast.svelte';
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
{#if sessionState.activePopup && sessionState.activePopup !== 'threadOptions'}
  <div
    transition:fade={{ duration: 150 }}
    class="overlay"
    onclick={() => (sessionState.activePopup = null)}
  >
    {#if sessionState.activePopup === 'folderSelector'}
      <Modal title={$t('settings.title')} size="xl">
        <SettingsView />
      </Modal>
    {:else if sessionState.activePopup === 'noteBrowser'}
      <Modal
        title={$t('notes.list.title')}
        size={settings.notesListLayout === 'masonry' ? 'xl' : 'md'}
        showLayoutToggle={true}
      >
        <NoteBrowser />
      </Modal>
    {:else if sessionState.activePopup === 'search'}
      <Modal
        title={$t('search.title')}
        size={settings.notesListLayout === 'masonry' ? 'xl' : 'lg'}
        showLayoutToggle={true}
      >
        <SearchNotes />
      </Modal>
    {:else if sessionState.activePopup === 'tagManager'}
      <Modal
        title={$t('tag.manager.title')}
        size={settings.notesListLayout === 'masonry' ? 'xl' : 'md'}
        showLayoutToggle={true}
      >
        <TagManagerModal />
      </Modal>
    {:else if sessionState.activePopup === 'shortcuts'}
      <Modal title={$t('shortcuts.title')} size="lg">
        <ShortcutListModal />
      </Modal>
    {:else if sessionState.activePopup === 'threadAggregation'}
      <Modal title={sessionState.aggregatedThread?.threadName} size="xl">
        <ThreadAggregationView />
      </Modal>
    {:else if sessionState.activePopup === 'about'}
      <Modal title="About TodayNote" size="md">
        <AboutView />
      </Modal>
    {:else if sessionState.activePopup === 'statistics'}
      <Modal title={$t('statistics.title')} size="lg">
        <StatisticsView />
      </Modal>
    {/if}
  </div>
{/if}

{#if sessionState.activePopup === 'threadOptions' && sessionState.selectedThreadForOptions}
  <ThreadOptionsMenu thread={sessionState.selectedThreadForOptions} />
{/if}

<div class="toast-container">
  {#each toast.toasts as item (item.id)}
    <div animate:flip={{ duration: 200 }}>
      <Toast t={item} />
    </div>
  {/each}
</div>

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: grid;
    place-items: center;
    z-index: 2000;
    backdrop-filter: blur(0.25rem);
  }

  .toast-container {
    position: fixed;
    bottom: 1.5rem;
    right: 1.5rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    pointer-events: none;
    gap: 0.5rem;
  }
</style>
