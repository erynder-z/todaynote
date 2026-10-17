<script lang="ts">
  import { KeyboardShortcut } from '#lib';
  /**
   * Component for displaying a single thread's content.
   */
  import { sessionState } from '#lib/stores/sessionState.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { t } from '#lib/utils/i18n.ts';
  import { notesService } from '#lib/utils/notes.ts';
  import { useShortcuts } from '#lib/utils/shortcuts.ts';
  import { linkOpenerPlugin } from '../plugins/linkOpenerPlugin.ts';
  import MilkdownEditor from './MilkdownEditor.svelte';

  const content = $derived(sessionState.threadViewContent);
  const threadId = $derived(sessionState.threadViewThreadId);
  const filename = $derived(sessionState.threadViewFilename);
  const editorPlugins = $derived([linkOpenerPlugin]);

  /**
   * Unpins the current thread.
   */
  const handleUnpinThread = async () => {
    if (!threadId || !filename) return;
    if (!settings.notesFolder) return;

    try {
      // Construct the full path to the note file
      const path = `${settings.notesFolder}/${filename}`;

      // Get the full note content
      const noteContent = await notesService.readNoteContent(path);
      if (!noteContent) {
        toast.error($t('notes.error.load'));
        return;
      }

      // Toggle the pin status (which will unpin it)
      const result = await notesService.toggleThreadPin(
        threadId,
        noteContent.content,
      );

      if (result) {
        // Update the session state with the new content if the note is open
        const newContent = JSON.parse(JSON.stringify(result));
        if (
          !sessionState.todayNotePath ||
          sessionState.todayNotePath === path ||
          sessionState.todayNotePath.endsWith(filename)
        )
          sessionState.todayNoteContent = newContent;

        // Update the selected thread for options if it's the same thread
        const updatedThread = result.threads.find((t) => t.id === threadId);
        if (updatedThread)
          sessionState.selectedThreadForOptions = updatedThread;

        toast.success($t('shortcuts.thread.unpin_success'));
        // Close the thread view
        sessionState.activePopup = null;
      } else {
        toast.error($t('thread.options.pin_error'));
      }
    } catch (error) {
      console.error('Error unpinning thread:', error);
      toast.error($t('thread.options.pin_error'));
    }
  };

  useShortcuts({
    unpinThread: handleUnpinThread,
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="thread-view-container">
  <div class="thread-view-header">
    <button
      class="unpin-button"
      onclick={handleUnpinThread}
      title={$t('thread.options.unpin')}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1rem"
        viewBox="0 -960 960 960"
        width="1rem"
        fill="currentColor"
        ><path
          d="m640-480 80 80v80H520v240l-40 40-40-40v-240H240v-80l80-80v-280h-40v-80h400v80h-40v280Z"
        /></svg
      ><span class="shortcut-hint">
        <KeyboardShortcut primary key="U" />
      </span>
    </button>
  </div>
  {#if content}
    <div class="editor-wrapper">
      <MilkdownEditor {content} readonly plugins={editorPlugins} />
    </div>
  {:else}
    <div class="empty-state">
      <p class="muted">{$t('pinned_threads.no_content')}</p>
    </div>
  {/if}
</div>

<style>
  .thread-view-container {
    padding: 0 1rem;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .thread-view-header {
    position: absolute;
    top: 1.25rem;
    right: 1.75rem;
    z-index: 10;
  }

  .unpin-button {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background-color: color-mix(in srgb, var(--accent), transparent 90%);
    border: none;
    cursor: pointer;
    color: var(--text-muted);
    padding: 0.25rem;
    border-radius: 0.25rem;
    transition: all 0.2s ease;
  }

  .unpin-button:hover {
    color: var(--accent-hover);
  }

  .unpin-button svg {
    display: block;
  }

  .editor-wrapper {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem 0;
  }

  /* Override default Milkdown min-height and padding */
  .editor-wrapper :global(.milkdown) {
    min-height: auto !important;
    height: 100%;
  }

  .editor-wrapper :global(.milkdown .editor) {
    padding-bottom: 2rem;
  }

  .empty-state {
    display: flex;
    justify-content: center;
    align-items: center;
    flex: 1;
    color: var(--text-muted);
    font-style: italic;
  }

  .muted {
    opacity: 0.7;
  }
</style>
