<script lang="ts">
  import { settings, t } from '$lib';
  import { selectFolder } from '$lib/utils/folder';

  let selectedFolderPath: string | null = $state(null);

  let isUseFolderButtonEnabled = $derived(
    selectedFolderPath !== null &&
      selectedFolderPath !== $settings.notes_folder,
  );

  const handleSelectFolder = async () => {
    const path = await selectFolder();

    if (path) {
      selectedFolderPath = path;
    }
  };

  const handleUseFolder = async () => {
    if (selectedFolderPath) {
      await settings.switchNotesFolder(selectedFolderPath);
      selectedFolderPath = null;
    }
  };
</script>

<div class="setting-item">
  <label for="folder-select">{$t('settings.folder.title')}</label>
  <div class="button-container">
    <button onclick={handleSelectFolder} class="folder-button">
      {$t('settings.folder.select')}
    </button>

    <button
      onclick={handleUseFolder}
      class="folder-button use-folder-button"
      disabled={!isUseFolderButtonEnabled}
    >
      {$t('settings.save')}
    </button>
  </div>

  {#if selectedFolderPath && selectedFolderPath !== $settings.notes_folder}
    <p class="folder-path">{selectedFolderPath}</p>
  {:else if $settings.notes_folder}
    <p class="folder-path">
      {$t('settings.folder.current')}
      {$settings.notes_folder}
    </p>
  {:else}
    <p class="folder-path">{$t('settings.folder.no_folder')}</p>
  {/if}
</div>

<style>
  .setting-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    width: 100%;
  }

  label {
    font-weight: 600;
    font-size: 1.1rem;
  }

  .button-container {
    display: flex;
    gap: 0.5rem;
  }

  .folder-button {
    padding: 0.8rem 1.5rem;
    background-color: #646cff;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      background-color 0.2s,
      transform 0.1s;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .folder-button:hover:not(:disabled) {
    background-color: #535bf2;
  }

  .folder-button:active:not(:disabled) {
    transform: scale(0.98);
  }

  .folder-button:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
    box-shadow: none;
  }

  .folder-path {
    max-width: 80%;
    word-break: break-all;
    text-align: center;
    padding: 0.5rem 1rem;
    background-color: rgba(0, 0, 0, 0.05);
    border-radius: 6px;
    border: 1px solid rgba(0, 0, 0, 0.1);
  }

  .use-folder-button {
    background-color: #28a745;
  }

  .use-folder-button:hover:not(:disabled) {
    background-color: #218838;
  }

  @media (prefers-color-scheme: dark) {
    .folder-button {
      background-color: #747bff;
    }

    .folder-button:hover:not(:disabled) {
      background-color: #646cff;
    }

    .folder-path {
      background-color: rgba(255, 255, 255, 0.1);
      border: 1px solid rgba(255, 255, 255, 0.2);
    }

    .use-folder-button {
      background-color: #2ed14d;
    }

    .use-folder-button:hover:not(:disabled) {
      background-color: #28a745;
    }
  }
</style>
