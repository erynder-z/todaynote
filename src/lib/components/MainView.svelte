<script lang="ts">
  /**
   * Primary view orchestrator that decides whether to show the editor,
   * the welcome screen (for first-time setup), or a loading state.
   */
  import { FolderSelector, NoteDisplay, sessionState, settings } from '$lib';
</script>

{#if settings.notesFolder === ''}
  <div class="welcome-screen">
    <h1>Welcome to todaynote</h1>
    <p>Please select a folder!</p>
    <FolderSelector />
  </div>
{:else if sessionState.todayNoteContent}
  <NoteDisplay
    noteContent={sessionState.todayNoteContent}
    notePath={sessionState.todayNotePath}
  />
{:else}
  <p class="loading-text">Preparing your daily note...</p>
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
