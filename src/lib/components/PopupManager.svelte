<script lang="ts">
  /**
   * Orchestrates which modal is currently displayed based on
   * the `sessionState.activePopup` state.
   */

  import { flip } from 'svelte/animate';
  import {
    Modal,
    NotesList,
    SearchNotes,
    SettingsView,
    ShortcutListModal,
    sessionState,
    settings,
    TagManagerModal,
    t,
  } from '$lib';
  import { toast } from '$lib/stores/toast.svelte';
  import Toast from './Toast.svelte';
</script>

{#if sessionState.activePopup === 'folderSelector'}
  <Modal title={$t('settings.title')}>
    <SettingsView />
  </Modal>
{:else if sessionState.activePopup === 'notesList'}
  <Modal
    title={$t('notes.list.title')}
    wide={settings.notesListLayout === 'masonry'}
  >
    <NotesList />
  </Modal>
{:else if sessionState.activePopup === 'search'}
  <Modal title={$t('search.title')}>
    <SearchNotes />
  </Modal>
{:else if sessionState.activePopup === 'tagManager'}
  <Modal title={$t('tag.manager.title')}>
    <TagManagerModal />
  </Modal>
{:else if sessionState.activePopup === 'shortcuts'}
  <Modal title={$t('shortcuts.title')}>
    <ShortcutListModal />
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
