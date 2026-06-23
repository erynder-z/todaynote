<script lang="ts">
  /**
   * Component for choosing the custom default thread name for new notes.
   */
  import { invoke } from '@tauri-apps/api/core';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { sessionState, settings, t, toast } from '$lib';
  import { readNoteContent } from '../utils/notes';

  let localValue = $state(settings.defaultThreadName || '');

  /**
   * Derived value that shows the effective thread name (custom or localized default).
   */
  let effectiveValue = $derived(
    settings.defaultThreadName || $t('note.header'),
  );

  /**
   * Updates the global setting when the input changes.
   */
  const handleValueChange = async () => {
    const newValue = localValue.trim() || null;
    await settings.save({ ...settings, defaultThreadName: newValue });
  };

  /**
   * Applies the current default thread name to all existing notes after confirmation.
   */
  const handleApplyToAll = async () => {
    const name = effectiveValue;
    const confirmed = await ask(
      $t('settings.default_thread_name.confirm_message', { name }),
      {
        title: $t('settings.default_thread_name.confirm_title'),
        kind: 'warning',
        okLabel: $t('settings.default_thread_name.confirm_button'),
        cancelLabel: $t('settings.default_thread_name.cancel_button'),
      },
    );

    if (confirmed) {
      try {
        await invoke('apply_default_thread_name', { newName: name });
        toast.success($t('settings.default_thread_name.success'));

        // If a note is currently open, refresh it to show the changes
        if (sessionState.todayNotePath) {
          const updated = await readNoteContent(sessionState.todayNotePath);
          if (updated) sessionState.todayNoteContent = updated;
        }
      } catch (error) {
        console.error('Failed to apply thread name to all notes:', error);
        toast.error(String(error));
      }
    }
  };

  /**
   * Toggles whether new notes should be created without a default thread name.
   */
  const handleNoDefaultNameToggle = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const checked = target.checked;
    await settings.save({ ...settings, useDefaultThreadName: !checked });
  };

  /**
   * Sync local value with settings when settings change externally.
   */
  $effect(() => {
    localValue = settings.defaultThreadName || '';
  });
</script>

<div class="setting-item">
  <div class="name-container" class:disabled={!settings.useDefaultThreadName}>
    <label for="default-thread-name"
      >{$t('settings.default_thread_name.title')}</label
    >
    <div class="input-container">
      <input
        id="default-thread-name"
        type="text"
        class="name-input"
        placeholder={effectiveValue}
        bind:value={localValue}
        onchange={handleValueChange}
      />
      <button
        type="button"
        class="btn-primary"
        onclick={handleApplyToAll}
        title={$t('settings.default_thread_name.apply_to_all')}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.25rem"
          viewBox="0 -960 960 960"
          width="1.25rem"
          fill="currentColor"
          ><path
            d="m482-200 114-113-114-113-42 42 43 43q-28 1-54.5-9T381-381q-20-20-30.5-46T340-479q0-17 4.5-34t12.5-33l-44-44q-17 25-25 53t-8 57q0 38 15 75t44 66q29 29 65 43.5t74 15.5l-38 38 42 42Zm165-170q17-25 25-53t8-57q0-38-14.5-75.5T622-622q-29-29-65.5-43T482-679l38-39-42-42-114 113 114 113 42-42-44-44q27 0 55 10.5t48 30.5q20 20 30.5 46t10.5 52q0 17-4.5 34T603-414l44 44ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z"
          /></svg
        >
        {$t('settings.default_thread_name.apply_to_all')}
      </button>
    </div>
  </div>

  <div class="checkbox-container">
    <input
      type="checkbox"
      id="no-default-name"
      checked={!settings.useDefaultThreadName}
      onchange={handleNoDefaultNameToggle}
    />
    <label for="no-default-name"
      >{$t('settings.default_thread_name.none')}</label
    >
  </div>
  <div class="explanation">
    <span
      >{settings.useDefaultThreadName
        ? $t('settings.default_thread_name.explanation.default')
        : $t('settings.default_thread_name.explanation.none')}</span
    >
  </div>
</div>

<style>
  .setting-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    width: 100%;
  }

  .input-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
  }

  .name-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
  }

  .name-input {
    padding: 0.65rem 1rem;
    border-radius: 0.5rem;
    border: none;
    background: color-mix(in srgb, var(--accent), transparent 90%);
    color: var(--text-main);
    width: 25ch;
    font-size: 0.95rem;
    outline: none;
    transition: background 0.2s ease;
  }

  .name-input:focus {
    background: color-mix(in srgb, var(--accent), transparent 85%);
  }

  .name-input::placeholder {
    color: color-mix(in srgb, var(--text-main), transparent 50%);
    color: var(--text-main);
  }

  .btn-primary {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.4rem;
    padding: 0.5rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      opacity 0.15s cubic-bezier(0.2, 0, 0, 1),
      transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
    color: var(--accent-text);
    min-width: 12ch;
    background-color: var(--accent);
  }

  .btn-primary:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .btn-primary:active {
    transform: translateY(0);
  }

  .checkbox-container {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    cursor: pointer;
    border-top: 1px solid var(--border);
    margin-top: 0.4rem;
    padding-top: 0.8rem;
  }

  .name-container.disabled {
    opacity: 0.5;
  }

  .explanation {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-align: center;
  }

  input[type='checkbox'] {
    width: 1.2rem;
    height: 1.2rem;
    cursor: pointer;
    accent-color: var(--accent);
  }

  label {
    font-weight: 600;
    color: var(--text-main);
    cursor: pointer;
    user-select: none;
    text-align: center;
  }
</style>
