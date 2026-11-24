<script lang="ts">
  /**
   * Backup & Restore card. Exports the notes folder to a (optionally encrypted)
   * zip/tnbk archive and imports notes back. Imports ask via an OS dialog
   * whether existing notes should be overwritten; encrypted backups show a
   * password prompt first.
   */
  import { invoke } from '@tauri-apps/api/core';
  import { message, open, save } from '@tauri-apps/plugin-dialog';
  import { tick } from 'svelte';
  import { sessionState, t, toast } from '#lib';
  import { notesService } from '#lib/utils/notes.ts';

  type PasswordMode = 'export' | 'import' | null;

  let busy = $state(false);
  let passwordMode = $state<PasswordMode>(null);
  let pendingPath = $state<string | null>(null);
  let password = $state('');
  let passwordConfirm = $state('');
  let passwordError = $state('');
  let passwordInput: HTMLInputElement | null = $state(null);

  /**
   * Derives a default filename for the save dialog based on today's date.
   */
  const defaultBackupName = () => {
    const d = new Date();
    const iso = d.toISOString().slice(0, 10);
    return `todaynote-backup-${iso}`;
  };

  /**
   * Exports a plain (unencrypted) zip backup.
   */
  const handleExport = async () => {
    const path = await save({
      title: $t('settings.backup.export'),
      defaultPath: `${defaultBackupName()}.zip`,
      filters: [{ name: 'Zip archive', extensions: ['zip'] }],
    });
    if (!path) return;
    await runExport(path, null);
  };

  /**
   * Exports an encrypted .tnbk backup; prompts for a password first.
   */
  const handleExportEncrypted = async () => {
    const path = await save({
      title: $t('settings.backup.export_encrypted'),
      defaultPath: `${defaultBackupName()}.tnbk`,
      filters: [{ name: 'Encrypted backup', extensions: ['tnbk'] }],
    });
    if (!path) return;
    pendingPath = path;
    password = '';
    passwordConfirm = '';
    passwordError = '';
    passwordMode = 'export';
    await focusPassword();
  };

  /**
   * Imports notes from a chosen archive. If the file is encrypted, a password
   * prompt is shown first.
   */
  const handleImport = async () => {
    const path = await open({
      title: $t('settings.backup.import'),
      multiple: false,
      filters: [{ name: 'Backups', extensions: ['zip', 'tnbk'] }],
    });
    if (!path || typeof path !== 'string') return;

    const isEncrypted = path.toLowerCase().endsWith('.tnbk');
    if (isEncrypted) {
      pendingPath = path;
      password = '';
      passwordConfirm = '';
      passwordError = '';
      passwordMode = 'import';
      await focusPassword();
    } else {
      await startImport(path, null);
    }
  };

  /**
   * Asks via an OS dialog whether existing notes should be overwritten,
   * then runs the import with the chosen mode. Cancel (or dismissing the
   * dialog) aborts the import.
   *
   * Note: with custom button labels the dialog resolves with the pressed
   * button's *label* (localized), not a canonical 'Yes'/'No'/'Cancel' key,
   * so answers are matched against the labels passed in.
   */
  const startImport = async (path: string, password: string | null) => {
    const yesLabel = $t('settings.backup.overwrite_yes');
    const noLabel = $t('settings.backup.overwrite_no');
    const cancelLabel = $t('settings.backup.cancel');

    const answer = await message($t('settings.backup.overwrite_question'), {
      title: $t('settings.backup.import'),
      kind: 'warning',
      buttons: {
        yes: yesLabel,
        no: noLabel,
        cancel: cancelLabel,
      },
    });

    if (answer === yesLabel) {
      await runImport(path, password, true);
    } else if (answer === noLabel) {
      await runImport(path, password, false);
    }
    // Any other result (cancel button or dialog dismissed) aborts the import.
  };

  const focusPassword = async () => {
    await tick();
    passwordInput?.focus();
  };

  /**
   * Validates the entered password and runs the pending export or import.
   */
  const confirmPassword = async () => {
    if (!password) {
      passwordError = $t('settings.backup.password_empty');
      return;
    }
    if (passwordMode === 'export' && password !== passwordConfirm) {
      passwordError = $t('settings.backup.password_mismatch');
      return;
    }
    const path = pendingPath;
    const pw = password;
    const mode = passwordMode;
    closePassword();
    if (mode === 'export' && path) {
      await runExport(path, pw);
    } else if (mode === 'import' && path) {
      await startImport(path, pw);
    }
  };

  const closePassword = () => {
    passwordMode = null;
    pendingPath = null;
    password = '';
    passwordConfirm = '';
    passwordError = '';
  };

  const runExport = async (path: string, password: string | null) => {
    busy = true;
    try {
      await invoke<number>('export_notes_archive', { path, password });
      toast.success($t('settings.backup.export_success'));
    } catch (error) {
      console.error('Export failed:', error);
      toast.error(String(error));
    } finally {
      busy = false;
    }
  };

  const runImport = async (
    path: string,
    password: string | null,
    overwrite: boolean,
  ) => {
    busy = true;
    try {
      const openPath = overwrite ? sessionState.todayNotePath : null;
      const contentBefore = openPath
        ? await notesService.readNoteContent(openPath)
        : null;

      const report = await invoke<{
        imported: number;
        skipped: number;
        errors: string[];
      }>('import_notes_archive', { path, password, overwrite });

      if (report.errors.length > 0) {
        toast.warning(
          $t('settings.backup.import_success_errors', {
            imported: report.imported,
            skipped: report.skipped,
            errors: report.errors.length,
          }),
        );
      } else {
        toast.success(
          $t('settings.backup.import_success', {
            imported: report.imported,
            skipped: report.skipped,
          }),
        );
      }
      // Signal the note browser to reload the list.
      sessionState.notesVersion += 1;

      if (openPath) {
        const contentAfter = await notesService.readNoteContent(openPath);
        if (contentAfter && contentAfter.content !== contentBefore?.content) {
          sessionState.todayNotePath = contentAfter.path;
          sessionState.todayNoteContent = contentAfter;
          sessionState.pendingThreadJump = null;
        }
      }
    } catch (error) {
      console.error('Import failed:', error);
      toast.error(String(error));
    } finally {
      busy = false;
    }
  };
</script>

{#snippet passwordPrompt()}
  <div
    class="pw-overlay"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <div class="pw-box">
      <h3>{$t('settings.backup.password')}</h3>
      {#if passwordMode === 'import'}
        <p class="pw-hint">{$t('settings.backup.encrypted_prompt')}</p>
      {/if}
      <form
        onsubmit={(e) => {
          e.preventDefault();
          confirmPassword();
        }}
      >
        <input
          bind:this={passwordInput}
          bind:value={password}
          type="password"
          class="pw-input"
          placeholder={$t('settings.backup.password_placeholder')}
          autocomplete="new-password"
        />
        {#if passwordMode === 'export'}
          <input
            bind:value={passwordConfirm}
            type="password"
            class="pw-input"
            placeholder={$t('settings.backup.password_confirm')}
            autocomplete="new-password"
          />
        {/if}
        {#if passwordError}
          <p class="pw-error">{passwordError}</p>
        {/if}
        <div class="pw-actions">
          <button type="button" class="btn-cancel" onclick={closePassword}>
            {$t('settings.backup.cancel')}
          </button>
          <button type="submit" class="btn-primary">
            {$t('settings.backup.confirm')}
          </button>
        </div>
      </form>
    </div>
  </div>
{/snippet}

<div class="setting-item">
  <span class="setting-label">{$t('settings.backup.title')}</span>
  <div class="button-row">
    <button
      type="button"
      class="btn-primary"
      onclick={handleExport}
      disabled={busy}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        height="1.25rem"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M12,18l4-4l-1.4-1.4L13,14.2V10H11v4.2L9.4,12.6L8,14l4,4ZM5,8V19H19V8H5ZM5,21q-0.83,0-1.41-0.59T3,19V6.52Q3,6.18 3.11,5.85t0.34-0.6L4.7,3.72Q4.98,3.38 5.39,3.19T6.25,3h11.5q0.45,0 0.86,0.19T19.3,3.72l1.25,1.53q0.23,0.27 0.34,0.6Q21,6.18 21,6.52V19q0,0.82-0.59,1.41T19,21H5ZM5.4,6H18.6L17.75,5H6.25L5.4,6ZM12,13.5Z"
        /></svg
      >
      {$t('settings.backup.export')}
    </button>
    <button
      type="button"
      class="btn-primary"
      onclick={handleExportEncrypted}
      disabled={busy}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        height="1.25rem"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M10.5,15h3l-0.57-3.23q0.5-0.25 0.79-0.72T14,10q0-0.83-0.59-1.41T12,8T10.59,8.59T10,10q0,0.57 0.29,1.05t0.79,0.72L10.5,15ZM12,22q-3.47-0.88-5.74-3.99T4,11.1V5l8-3l8,3v6.1q0,3.8-2.26,6.91T12,22Zm0-2.1q2.6-0.82 4.3-3.3T18,11.1V6.38L12,4.13L6,6.38V11.1q0,3.03 1.7,5.5T12,19.9ZM12,12Z"
        /></svg
      >
      {$t('settings.backup.export_encrypted')}
    </button>
    <button
      type="button"
      class="btn-primary"
      onclick={handleImport}
      disabled={busy}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        height="1.25rem"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M12,14q-0.82,0-1.41-0.59T10,12t0.59-1.41T12,10t1.41,0.59Q14,11.18 14,12t-0.59,1.41T12,14Zm0,7q-3.47,0-6.03-2.29Q3.43,16.43 3.05,13H5.1q0.35,2.6 2.31,4.3T12,19q2.93,0 4.96-2.04Q19,14.93 19,12q0-2.93-2.04-4.96T12,5Q10.28,5 8.78,5.8T6.25,8H9v2H3V4H5V6.35q1.28-1.6 3.11-2.47T12,3q1.88,0 3.51,0.71t2.85,1.93t1.93,2.85T21,12t-0.71,3.51t-1.93,2.85t-2.85,1.93T12,21Z"
        /></svg
      >
      {$t('settings.backup.import')}
    </button>
  </div>
</div>

{#if passwordMode}
  {@render passwordPrompt()}
{/if}

<style>
  .setting-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    width: 100%;
  }

  .setting-label {
    font-weight: 600;
    color: var(--text-main);
  }

  .button-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    justify-content: center;
  }

  .btn-primary {
    display: flex;
    gap: 0.25rem;
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    color: var(--accent-text);
    background-color: var(--accent);
    transition:
      opacity 0.15s ease,
      transform 0.15s ease;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .btn-primary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .pw-overlay {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    z-index: 999;
    backdrop-filter: brightness(60%);
  }

  :global(html.linux) .pw-overlay {
    backdrop-filter: none;
    background-color: rgb(0 0 0 / 0.5);
  }

  .pw-box {
    background-color: var(--bg-base);
    border-radius: 0.75rem;
    padding: 1.5rem;
    width: min(24rem, 90dvw);
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
    border: 1px solid var(--border);
  }

  .pw-box h3 {
    margin: 0 0 0.75rem;
    text-align: center;
    color: var(--text-main);
  }

  .pw-hint {
    font-size: 0.8rem;
    color: var(--text-muted);
    text-align: center;
    margin-bottom: 0.75rem;
  }

  .pw-input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5rem 0.65rem;
    margin-bottom: 0.5rem;
    border-radius: 0.5rem;
    border: 1px solid var(--border);
    background-color: var(--bg-surface);
    color: var(--text-main);
    font-size: 0.95rem;
  }

  .pw-error {
    color: var(--error);
    font-size: 0.8rem;
    margin: 0 0 0.5rem;
  }

  .pw-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .btn-cancel {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    background: none;
    color: var(--text-main);
    cursor: pointer;
    font-size: 0.9rem;
  }
</style>
