<script lang="ts">
  import {
    appState,
    Modal,
    NotesList,
    SearchNotes,
    SettingsView,
    t,
  } from '$lib';
</script>

{#if $appState.activePopup === 'folderSelector'}
  <Modal title={$t('settings.title')}>
    <SettingsView />
  </Modal>
{:else if $appState.activePopup === 'notesList'}
  <Modal title={$t('notes.list.title')}>
    <NotesList />
  </Modal>
{:else if $appState.activePopup === 'search'}
  <Modal title={$t('search.title')}>
    <SearchNotes />
  </Modal>
{/if}
