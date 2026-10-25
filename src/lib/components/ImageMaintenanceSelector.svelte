<script lang="ts">
  /**
   * Image maintenance card. Deletes images stored in the notes folder that
   * are not referenced by any note.
   */
  import { invoke } from '@tauri-apps/api/core';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { t, toast } from '#lib';
  import type { PurgeReport } from '#lib/types/ui.ts';

  let busy = $state(false);

  /**
   * Formats a byte count as a human-readable size.
   */
  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  };

  /**
   * Deletes all images not referenced by any note, after confirmation.
   */
  const handlePurge = async () => {
    const confirmed = await ask($t('settings.images.purge_confirm_message'), {
      title: $t('settings.images.purge_confirm_title'),
      kind: 'warning',
    });
    if (!confirmed) return;

    busy = true;
    try {
      const report = await invoke<PurgeReport>('purge_unused_images');
      toast.success(
        $t('settings.images.purge_success', {
          count: report.removed,
          freed: formatBytes(report.freedBytes),
        }),
      );
    } catch (error) {
      console.error('Failed to purge unused images:', error);
      toast.error(String(error));
    } finally {
      busy = false;
    }
  };
</script>

<div class="setting-item">
  <span class="setting-label">{$t('settings.images.title')}</span>
  <button type="button" class="btn-purge" onclick={handlePurge} disabled={busy}>
    {$t('settings.images.purge')}
  </button>
</div>

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

  .btn-purge {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    color: var(--bg-base);
    background-color: var(--error);
    transition:
      opacity 0.15s ease,
      transform 0.15s ease;
  }

  .btn-purge:hover:not(:disabled) {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .btn-purge:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
