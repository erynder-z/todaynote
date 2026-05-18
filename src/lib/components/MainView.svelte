<script lang="ts">
  /**
   * Primary view orchestrator that decides whether to show the editor,
   * the welcome screen (for first-time setup), or a loading state.
   */

  import { sessionState } from '../stores/sessionState.svelte';
  import { settings } from '../stores/settings.svelte';
  import { t } from '../utils/i18n';
  import EditorView from './EditorView.svelte';
  import FolderSelector from './FolderSelector.svelte';
</script>

{#if settings.notesFolder === ''}
  <div class="welcome-screen">
    <p>{$t('welcome.no_folder')}</p>
    <FolderSelector />
  </div>
{:else if sessionState.todayNoteContent}
  <EditorView
    bind:noteContent={sessionState.todayNoteContent}
    notePath={sessionState.todayNotePath}
  />
{:else}
  <p class="loading-text">{$t('welcome.loading_note')}</p>
{/if}

<style>
  .welcome-screen {
    text-align: center;
    margin-top: 15vh;
  }

  .loading-text {
    text-align: center;
    margin-top: 20vh;
    color: var(--text-muted);
  }
</style>
