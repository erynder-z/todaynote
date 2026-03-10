<script lang="ts">
  /**
   * Provides a UI for selecting and validating a directory as the root for notes.
   */
  import { invoke } from '@tauri-apps/api/core';
  import { settings, t } from '$lib';
  import type { FolderValidation } from '$lib/types/folder';
  import { selectFolder } from '$lib/utils/folder';

  let selectedFolderPath = $state<string | null>(null);
  let validationResult = $state<FolderValidation | null>(null);

  /**
   * Derived boolean indicating if the currently selected folder is valid and can be applied.
   */
  let isUseFolderButtonEnabled = $derived(
    selectedFolderPath !== null &&
      validationResult !== null &&
      validationResult.isValid &&
      validationResult.isWritable &&
      selectedFolderPath !== settings.notesFolder,
  );

  /**
   * Opens the file dialog and triggers backend validation for the selected path.
   */
  const handleSelectFolder = async () => {
    const path = await selectFolder();

    if (path) {
      selectedFolderPath = path;
      try {
        const result: FolderValidation = await invoke('validate_folder', {
          path,
        });
        validationResult = result;
      } catch (error) {
        console.error('Validation failed:', error);
        validationResult = {
          isValid: false,
          isWritable: false,
          exists: false,
          noteCount: 0,
          error: String(error),
        };
      }
    }
  };

  /**
   * Finalizes the folder selection and updates the application settings.
   */
  const handleUseFolder = async () => {
    if (selectedFolderPath && isUseFolderButtonEnabled) {
      await settings.switchNotesFolder(selectedFolderPath);
      selectedFolderPath = null;
      validationResult = null;
    }
  };
</script>

<div class="setting-item">
  <label for="folder-select">{$t('settings.folder.title')}</label>
  <div class="button-container">
    <button onclick={handleSelectFolder} class="btn-primary">
      {$t('settings.folder.select')}
    </button>

    <button
      onclick={handleUseFolder}
      class="btn-success"
      disabled={!isUseFolderButtonEnabled}
    >
      {$t('settings.save')}
    </button>
  </div>

  <div class="folder-status">
    {#if selectedFolderPath}
      <p class="folder-path">
        <strong>{$t('settings.folder.selected')}</strong>
        <span class="path-text">{selectedFolderPath}</span>
      </p>
      {#if validationResult}
        <p
          class="validation-msg"
          class:error={!validationResult.isValid ||
            !validationResult.isWritable}
        >
          {#if !validationResult.isValid || !validationResult.isWritable}
            {$t('settings.folder.validation.invalid', {
              error: validationResult.error || 'Unknown error',
            })}
          {:else if validationResult.exists}
            {$t('settings.folder.validation.valid_existing', {
              count: validationResult.noteCount,
            })}
          {:else}
            {$t('settings.folder.validation.valid_new')}
          {/if}
        </p>
      {/if}
    {:else if settings.notesFolder}
      <p class="folder-path">
        <strong>{$t('settings.folder.current')}</strong>
        <span class="path-text">{settings.notesFolder}</span>
      </p>
    {:else}
      <p class="folder-path">{$t('settings.folder.no_folder')}</p>
    {/if}
  </div>
</div>

<style>
  .setting-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
    width: 100%;
  }

  label {
    font-weight: 700;
    color: var(--text-main);
  }

  .button-container {
    display: flex;
    gap: 0.75rem;
  }

  .btn-primary,
  .btn-success {
    padding: 0.8rem 1.75rem; /* Wider padding for monospace buttons */
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
    color: var(--accent-text);
    min-width: 15ch;
  }

  .btn-primary {
    background-color: var(--accent);
  }
  .btn-primary:hover {
    background-color: var(--accent-hover);
  }

  .btn-success {
    background-color: var(--success);
  }

  .btn-primary:disabled,
  .btn-success:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .folder-status {
    width: 100%;
    max-width: 65ch;
  }

  .folder-path {
    text-align: center;
    padding: 1rem;
    background-color: var(--bg-surface);
    border-radius: 0.375rem;
    border: 0.0625rem solid var(--border);
    color: var(--text-main);
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .path-text {
    font-size: 0.85rem;
    overflow-wrap: break-word;
    opacity: 0.9;
  }

  .validation-msg {
    font-size: 0.85rem;
    color: var(--success);
    margin-top: 0.5rem;
    text-align: center;
  }

  .validation-msg.error {
    color: var(--error);
  }
</style>
