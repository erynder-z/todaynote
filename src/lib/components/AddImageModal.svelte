<script lang="ts">
  /**
   * Modal for adding an image to the selected thread.
   * The image file is stored in the notes images directory and the note
   * receives a relative markdown reference.
   */
  import { open } from '@tauri-apps/plugin-dialog';
  import { sessionState } from '../stores/sessionState.svelte.ts';
  import { toast } from '../stores/toast.svelte.ts';
  import { t } from '../utils/i18n.ts';
  import { notesService } from '../utils/notes.ts';

  /** Extensions accepted by the backend for stored images (no SVG). */
  const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'avif'];

  let mode: 'choice' | 'url' = $state('choice');
  let url = $state('');
  let isWorking = $state(false);

  /**
   * Closes the modal.
   */
  const close = () => {
    sessionState.activePopup = null;
  };

  /**
   * Inserts the image into the line after the one the caret is in,
   * through the live editor document, so the change flows through the
   * regular editor update/auto-save path. No document replacement,
   * no cursor jump.
   */
  const insertImage = (relativePath: string) => {
    const service = sessionState.editorService;

    if (!service || !service.insertImage(relativePath)) {
      toast.error($t('image.insert_error'));
      return;
    }

    toast.success($t('image.success'));
    close();
  };

  /**
   * Opens the OS file picker and stores the selected image locally.
   */
  const handleLocal = async () => {
    if (isWorking) return;

    const selection = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Images', extensions: IMAGE_EXTENSIONS }],
    });

    if (typeof selection !== 'string') return;

    isWorking = true;
    const relative = await notesService.addImageFromPath(selection);
    isWorking = false;

    if (!relative) {
      toast.error($t('image.error'));
      return;
    }

    insertImage(relative);
  };

  /**
   * Downloads the entered image URL and stores it locally.
   */
  const handleRemote = async () => {
    const trimmed = url.trim();
    if (!trimmed || isWorking) return;

    isWorking = true;
    const relative = await notesService.addImageFromUrl(trimmed);
    isWorking = false;

    if (!relative) {
      toast.error($t('image.error'));
      return;
    }

    url = '';
    insertImage(relative);
  };
</script>

{#if mode === 'choice'}
  <div class="choice-container">
    <button class="choice-button" onclick={handleLocal} disabled={isWorking}>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M360-400h400L622-580l-92 120-62-80-108 140Zm-40 160q-33 0-56.5-23.5T240-320v-480q0-33 23.5-56.5T320-880h480q33 0 56.5 23.5T880-800v480q0 33-23.5 56.5T800-240H320Zm0-80h480v-480H320v480ZM160-80q-33 0-56.5-23.5T80-160v-560h80v560h560v80H160Zm160-720v480-480Z"
        /></svg
      >
      <span>{$t('image.local')}</span>
    </button>

    <button class="choice-button" onclick={() => (mode = 'url')}>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.25rem"
        viewBox="0 -960 960 960"
        width="1.25rem"
        fill="currentColor"
        ><path
          d="M260-160q-91 0-155.5-63T40-377q0-78 47-139t123-78q25-92 100-149t170-57q117 0 198.5 81.5T760-520q69 8 114.5 59.5T920-340q0 75-52.5 127.5T740-160H260Zm0-80h480q42 0 71-29t29-71q0-42-29-71t-71-29h-60v-80q0-83-58.5-141.5T480-720q-83 0-141.5 58.5T280-520h-20q-58 0-99 41t-41 99q0 58 41 99t99 41Zm220-240Z"
        /></svg
      >
      <span>{$t('image.remote')}</span>
    </button>
  </div>
{:else}
  <form
    class="url-form"
    onsubmit={(e) => {
      e.preventDefault();
      handleRemote();
    }}
  >
    <!-- svelte-ignore a11y_autofocus -->
    <input
      type="url"
      bind:value={url}
      placeholder={$t('image.url_placeholder')}
      spellcheck="false"
      autofocus
      disabled={isWorking}
    />
    <button type="submit" disabled={isWorking || !url.trim()}>
      {isWorking ? $t('image.loading') : $t('image.add')}
    </button>
  </form>
{/if}

<style>
  .choice-container {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .choice-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    padding: 1rem;
    background-color: color-mix(in srgb, var(--bg-surface), transparent 45%);
    border: 1px solid var(--border);
    color: var(--text-main);
    font-weight: 600;
    cursor: pointer;
    transition:
      border-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      color 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .choice-button:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
  }

  .choice-button:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .url-form {
    display: flex;
    gap: 0.5rem;
  }

  .url-form input {
    flex: 1;
    padding: 0.5rem 0.75rem;
    background: none;
    border: 1px solid var(--border);
    border-radius: 0.25rem;
    color: var(--text-main);
    font-size: 0.9rem;
    outline: none;
  }

  .url-form input:focus {
    border-color: var(--accent);
  }

  .url-form button {
    padding: 0.5rem 1rem;
    background-color: var(--accent);
    border: none;
    border-radius: 0.25rem;
    color: var(--accent-text);
    font-weight: 600;
    cursor: pointer;
  }

  .url-form button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
