<script lang="ts">
  /**
   * Orchestrates which modal is currently displayed based on
   * the `sessionState.activePopup` state.
   */
  import { flip } from 'svelte/animate';
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
{:else if sessionState.activePopup === 'threadOptions' && sessionState.selectedThreadForOptions}
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
