<script lang="ts">
  /**
   * Orchestrates which modal is currently displayed based on
   * the `sessionState.activePopup` state.
   */
  import {
    Modal,
    NotesList,
    SearchNotes,
    SettingsView,
    ShortcutListModal,
    sessionState,
    TagManagerModal,
    t,
  } from '$lib';
</script>

{#if sessionState.activePopup === 'folderSelector'}
  <Modal title={$t('settings.title')}>
    <SettingsView />
  </Modal>
{:else if sessionState.activePopup === 'notesList'}
  <Modal title={$t('notes.list.title')}>
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
