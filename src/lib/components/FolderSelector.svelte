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
      validationResult?.isValid &&
      validationResult?.isWritable &&
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
    padding: 0.65rem 1.25rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    color: var(--accent-text);
    min-width: 12ch;
  }

  .btn-primary {
    background-color: var(--accent);
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .btn-success {
    background-color: var(--success, #28a745);
  }

  .btn-success:hover:not(:disabled) {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .btn-primary:active:not(:disabled),
  .btn-success:active:not(:disabled) {
    transform: translateY(0);
  }

  .btn-primary:disabled,
  .btn-success:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .folder-status {
    width: 100%;
    max-width: 50ch;
  }

  .folder-path {
    text-align: center;
    padding: 1rem;
    background-color: var(--bg-surface);
    border-radius: 0.5rem;
    border: 0.1rem dashed var(--border);
    color: var(--text-main);
    margin: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .path-text {
    font-size: 0.85rem;
    font-family: var(--font-mono, monospace);
    overflow-wrap: break-word;
    opacity: 0.8;
  }

  .validation-msg {
    font-size: 0.85rem;
    color: var(--success, #28a745);
    margin-top: 0.75rem;
    text-align: center;
    font-weight: 500;
  }

  .validation-msg.error {
    color: var(--error);
  }
</style>
