<script lang="ts">
  /**
   * Maintenance card. Combines destructive cleanup actions: purging empty
   * notes and deleting images that are not referenced by any note.
   */
  import { invoke } from '@tauri-apps/api/core';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { t, toast } from '#lib';

  let busy = $state(false);

  /**
   * Purges all empty notes after confirmation.
   */
  const handlePurgeNotes = async () => {
    const confirmed = await ask(
      $t('settings.purge_empty_notes.confirm_message'),
      {
        title: $t('settings.purge_empty_notes.confirm_title'),
        kind: 'warning',
      },
    );
    if (!confirmed) return;

    busy = true;
    try {
      const purgedCount = await invoke<number>('purge_empty_notes');
      toast.success(
        $t('settings.purge_empty_notes.success', { count: purgedCount }),
      );
    } catch (error) {
      console.error('Failed to purge empty notes:', error);
      toast.error(String(error));
    } finally {
      busy = false;
    }
  };

  /**
   * Deletes all images not referenced by any note, after confirmation.
   */
  const handlePurgeImages = async () => {
    const confirmed = await ask($t('settings.images.purge_confirm_message'), {
      title: $t('settings.images.purge_confirm_title'),
      kind: 'warning',
    });
    if (!confirmed) return;

    busy = true;
    try {
      const removedCount = await invoke<number>('purge_unused_images');
      toast.success(
        $t('settings.images.purge_success', { count: removedCount }),
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
  <span class="setting-label">{$t('settings.maintenance.title')}</span>
  <div class="button-row">
    <button
      type="button"
      class="btn-purge"
      onclick={handlePurgeNotes}
      disabled={busy}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M240-800v200-200 640-9.5 9.5-640Zm0 720q-33 0-56.5-23.5T160-160v-640q0-33 23.5-56.5T240-880h320l240 240v174q-19-7-39-10.5t-41-3.5v-120H520v-200H240v640h254q8 23 20 43t28 37H240Zm396-20-56-56 84-84-84-84 56-56 84 84 84-84 56 56-83 84 83 84-56 56-84-83-84 83Z"
        /></svg
      >
      {$t('settings.purge_empty_notes.button')}
    </button>
    <button
      type="button"
      class="btn-purge"
      onclick={handlePurgeImages}
      disabled={busy}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M240-800v200-200 640-9.5 9.5-640Zm0 720q-33 0-56.5-23.5T160-160v-640q0-33 23.5-56.5T240-880h320l240 240v174q-19-7-39-10.5t-41-3.5v-120H520v-200H240v640h254q8 23 20 43t28 37H240Zm396-20-56-56 84-84-84-84 56-56 84 84 84-84 56 56-83 84 83 84-56 56-84-83-84 83Z"
        /></svg
      >
      {$t('settings.images.purge')}
    </button>
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

  .btn-purge {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.65rem 1.25rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      opacity 0.15s ease,
      transform 0.15s ease;
    color: var(--bg-base);
    background-color: var(--error);
  }

  .btn-purge:hover:not(:disabled) {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .btn-purge:active:not(:disabled) {
    transform: translateY(0);
  }

  .btn-purge:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
