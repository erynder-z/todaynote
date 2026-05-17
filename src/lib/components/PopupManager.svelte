<script lang="ts">
  /**
   * Orchestrates which modal is currently displayed based on
   * the `sessionState.activePopup` state.
   */

  import { flip } from 'svelte/animate';
  import {
    Modal,
    NoteBrowser,
    SearchNotes,
    SettingsView,
    ShortcutListModal,
    sessionState,
    settings,
    TagManagerModal,
    ThreadAggregationView,
    t,
  } from '$lib';
  import { toast } from '$lib/stores/toast.svelte';
  import Toast from './Toast.svelte';
</script>

{#if sessionState.activePopup === 'folderSelector'}
  <Modal title={$t('settings.title')} size="md">
    <SettingsView />
  </Modal>
{:else if sessionState.activePopup === 'noteBrowser'}
  <Modal
    title={$t('notes.list.title')}
    size={settings.notesListLayout === 'masonry' ? 'xl' : 'md'}
  >
    <NoteBrowser />
  </Modal>
{:else if sessionState.activePopup === 'search'}
  <Modal title={$t('search.title')} size="xl">
    <SearchNotes />
  </Modal>
{:else if sessionState.activePopup === 'tagManager'}
  <Modal title={$t('tag.manager.title')} size="sm">
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
