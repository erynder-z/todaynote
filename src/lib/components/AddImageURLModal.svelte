<script lang="ts">
  /**
   * Modal for adding an image to the note from a remote URL.
   * The image file is downloaded into the notes images directory and the
   * note receives a relative markdown reference at the cursor position.
   */
  import { sessionState } from '../stores/sessionState.svelte.ts';
  import { toast } from '../stores/toast.svelte.ts';
  import { t } from '../utils/i18n.ts';
  import { insertImage } from '../utils/imageInput.ts';
  import { notesService } from '../utils/notes.ts';

  let url = $state('');
  let isWorking = $state(false);

  /**
   * Closes the modal.
   */
  const close = () => {
    sessionState.activePopup = null;
  };

  /**
   * Downloads the entered image URL, stores it locally and inserts it
   * into the line after the one the cursor is in.
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
    if (insertImage(relative)) close();
  };
</script>

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

<style>
  .url-form {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
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
