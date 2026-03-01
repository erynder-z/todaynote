<script lang="ts">
  import { appState, FolderSelector, NoteDisplay, settings } from '$lib';

  let currentSettings = $derived($settings);
</script>

{#if currentSettings.notes_folder === null}
  <div class="welcome-screen">
    <h1>Welcome to todaynote</h1>
    <p>Please select a folder to start your daily notes journey.</p>
    <FolderSelector />
  </div>
{:else if $appState.todayNoteContent}
  <NoteDisplay noteContent={$appState.todayNoteContent} />
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
