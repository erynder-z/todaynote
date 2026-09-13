<script lang="ts">
  /**
   * Backup & Restore card. Exports the notes folder to a (optionally encrypted)
   * zip/tnbk archive and imports notes back, with an overwrite toggle and a
   * password prompt for encrypted backups.
   */
  import { invoke } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { tick } from 'svelte';
  import { sessionState, t, toast } from '$lib';

  type PasswordMode = 'export' | 'import' | null;

  let overwrite = $state(false);
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
      await runImport(path, null);
    }
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
      await runImport(path, pw);
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

  const runImport = async (path: string, password: string | null) => {
    busy = true;
    try {
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
      {$t('settings.backup.export')}
    </button>
    <button
      type="button"
      class="btn-primary"
      onclick={handleExportEncrypted}
      disabled={busy}
    >
      {$t('settings.backup.export_encrypted')}
    </button>
    <button
      type="button"
      class="btn-primary"
      onclick={handleImport}
      disabled={busy}
    >
      {$t('settings.backup.import')}
    </button>
  </div>

  <label class="overwrite-toggle">
    <input type="checkbox" bind:checked={overwrite} disabled={busy} />
    <span>{$t('settings.backup.overwrite')}</span>
  </label>
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

  .overwrite-toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
    color: var(--text-muted);
    cursor: pointer;
  }

  .pw-overlay {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    z-index: 999;
    backdrop-filter: brightness(60%);
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
