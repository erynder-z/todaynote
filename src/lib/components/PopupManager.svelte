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
    sessionState,
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
{/if}
