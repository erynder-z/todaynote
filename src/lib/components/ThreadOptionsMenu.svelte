<script lang="ts">
  /**
   * Slide-in menu for displaying thread options
   */
  import { slide } from 'svelte/transition';
  import type { NoteThread } from '#lib/interfaces/notes.js';
  import { toast } from '#lib/stores/toast.svelte.js';
  import { t } from '#lib/utils/i18n.js';
  import { stripMarkdown } from '#lib/utils/markdown.js';
  import { sessionState } from '../stores/sessionState.svelte';
  import { notesService } from '../utils/notes';
  import { useShortcuts } from '../utils/shortcuts';
  import ThreadOptionButton from './ThreadOptionButton.svelte';

  interface Props {
    thread: NoteThread;
  }

  let { thread }: Props = $props();

  let currentThread = $derived(
    sessionState.todayNoteContent?.threads.find((t) => t.id === thread.id) ??
      thread,
  );

  let aggregatedThread = $state<Awaited<
    ReturnType<typeof notesService.aggregateThread>
  > | null>(null);

  let hasLinkedThreads = $state(false);

  /**
   * Whether the thread has any non-empty content lines (beyond the header)
   */
  let hasContent = $derived.by(() => {
    const content = sessionState.todayNoteContent?.content;
    if (!content) return false;

    const lines = content.split('\n');
    // Content lines are between startLine + 1 (skip !!! header) and endLine
    const contentLines = lines.slice(
      currentThread.startLine + 1,
      currentThread.endLine,
    );
    return contentLines.some((line) => line.trim().length > 0);
  });

  /**
   * Closes the menu
   */
  const closeMenu = () => {
    sessionState.activePopup = null;
  };

  /**
   * Loads linked thread information
   */
  const loadLinkedThreads = async () => {
    try {
      aggregatedThread = await notesService.aggregateThread(currentThread.name);
      hasLinkedThreads =
        aggregatedThread !== null && aggregatedThread.items.length > 1;
    } catch {
      aggregatedThread = null;
      hasLinkedThreads = false;
    }
  };

  /**
   * Opens linked threads popup
   */
  const handleLinked = () => {
    if (!aggregatedThread || !hasLinkedThreads) {
      return;
    }

    sessionState.aggregatedThread = aggregatedThread;
    sessionState.activePopup = 'threadAggregation';
  };

  /**
   * Copies the thread content to the clipboard after stripping the markdown from it
   */
  const handleCopyThread = async () => {
    try {
      const content = sessionState.todayNoteContent?.content;

      if (!content || !hasContent) return;

      const lines = content.split('\n');

      const threadContent = lines
        .slice(currentThread.startLine + 1, currentThread.endLine)
        .join('\n');

      const plainText = stripMarkdown(threadContent);

      await navigator.clipboard.writeText(plainText);

      toast.success($t('thread.options.copy_success'));

      closeMenu();
    } catch {
      toast.error($t('thread.options.copy_error'));
    }
  };

  /**
   * Toggles the pin status of the current thread
   */
  const handleTogglePin = async () => {
    try {
      // Do nothing if the thread has no content
      if (!hasContent) return;

      const currentContent = sessionState.todayNoteContent?.content;

      if (!currentContent) {
        toast.error($t('thread.options.pin_error'));
        return;
      }

      const result = await notesService.toggleThreadPin(
        currentThread.id,
        currentContent,
      );

      // Update the session state with the new content
      if (result) {
        // Force a refresh by creating a new object with the same data
        const newContent = JSON.parse(JSON.stringify(result));
        sessionState.todayNoteContent = newContent;
        const updatedThread = result.threads.find(
          (t) => t.id === currentThread.id,
        );
        if (updatedThread)
          sessionState.selectedThreadForOptions = updatedThread;
      }
    } catch {
      toast.error($t('thread.options.pin_error'));
    }
  };

  /**
   * Deletes the current thread
   */
  const handleRemoveThread = async () => {
    try {
      const currentContent = sessionState.todayNoteContent?.content;

      if (!currentContent) {
        toast.error($t('thread.options.remove_error'));
        return;
      }

      const result = await notesService.removeThread(
        currentThread.id,
        currentContent,
      );

      // Update the session state with the new content
      if (result) {
        // Force a refresh by creating a new object with the same data
        const newContent = JSON.parse(JSON.stringify(result));
        sessionState.todayNoteContent = newContent;
      }

      closeMenu();
    } catch (error) {
      toast.error($t('thread.options.remove_error'));
    }
  };

  $effect(() => {
    currentThread.name;
    loadLinkedThreads();
  });

  useShortcuts('thread-options', {
    threadOptionRemove: handleRemoveThread,
    threadOptionLinked: handleLinked,
    threadOptionCopy: handleCopyThread,
    threadOptionPin: handleTogglePin,
    closePopup: closeMenu,
  });
</script>

<div
  class="thread-options-taskbar"
  transition:slide={{ duration: 150, axis: 'y' }}
>
  <div class="taskbar-content">
    <div class="thread-info">
      <h3 class="thread-name">{currentThread.name}</h3>
      <button class="close-button" onclick={closeMenu} aria-label="Close">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.2rem"
          viewBox="0 -960 960 960"
          width="1.2rem"
          fill="currentColor"
        >
          <path
            d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
          />
        </svg>
      </button>
    </div>

    <div class="taskbar-actions">
      <ThreadOptionButton
        label={currentThread.pinned
          ? $t('thread.options.unpin')
          : $t('thread.options.pin')}
        title={currentThread.pinned
          ? $t('thread.options.unpin')
          : $t('thread.options.pin')}
        shortcutKey="P"
        active={currentThread.pinned}
        disabled={!hasContent}
        onclick={handleTogglePin}
      >
        {#if currentThread.pinned}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="1.2rem"
            viewBox="0 -960 960 960"
            width="1.2rem"
            fill="currentColor"
            ><path
              d="m640-480 80 80v80H520v240l-40 40-40-40v-240H240v-80l80-80v-280h-40v-80h400v80h-40v280Z"
            /></svg
          >
        {:else}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="1.2rem"
            viewBox="0 -960 960 960"
            width="1.2rem"
            fill="currentColor"
            ><g
              data-fr-wrap="1"
              transform="translate(480 -480) rotate(45) translate(-480 480)"
              ><path
                d="m640-480 80 80v80H520v240l-40 40-40-40v-240H240v-80l80-80v-280h-40v-80h400v80h-40v280Zm-286 80h252l-46-46v-314H400v314l-46 46Zm126 0Z"
              /></g
            ></svg
          >
        {/if}
      </ThreadOptionButton>

      <ThreadOptionButton
        label={hasLinkedThreads
          ? $t('thread.options.linked')
          : $t('thread.options.no_linked')}
        title={$t('thread.options.linked')}
        shortcutKey={hasLinkedThreads ? 'I' : ''}
        disabled={!hasLinkedThreads}
        onclick={handleLinked}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 -960 960 960"
          fill="currentColor"
          ><path
            d="M760-600q-57 0-99-34t-56-86H354q-11 42-41.5 72.5T240-606v251q52 14 86 56t34 99q0 66-47 113T200-40q-66 0-113-47T40-200q0-57 34-99t86-56v-251q-52-14-86-56t-34-98q0-66 47-113t113-47q56 0 98 34t56 86h251q14-52 56-86t99-34q66 0 113 47t47 113q0 66-47 113t-113 47ZM200-120q33 0 56.5-24t23.5-56q0-33-23.5-56.5T200-280q-32 0-56 23.5T120-200q0 32 24 56t56 24Zm0-560q33 0 56.5-23.5T280-760q0-33-23.5-56.5T200-840q-32 0-56 23.5T120-760q0 33 24 56.5t56 23.5ZM760-40q-66 0-113-47t-47-113q0-66 47-113t113-47q66 0 113 47t47 113q0 66-47 113T760-40Zm0-80q33 0 56.5-24t23.5-56q0-33-23.5-56.5T760-280q-33 0-56.5 23.5T680-200q0 32 23.5 56t56.5 24Zm0-560q33 0 56.5-23.5T840-760q0-33-23.5-56.5T760-840q-33 0-56.5 23.5T680-760q0 33 23.5 56.5T760-680ZM200-200Zm0-560Zm560 560Zm0-560Z"
          /></svg
        >
      </ThreadOptionButton>

      <ThreadOptionButton
        label={hasContent
          ? $t('thread.options.copy')
          : $t('thread.options.no_content')}
        title={$t('thread.options.copy')}
        shortcutKey={hasContent ? 'C' : ''}
        disabled={!hasContent}
        onclick={handleCopyThread}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.2rem"
          viewBox="0 -960 960 960"
          width="1.2rem"
          fill="currentColor"
          ><path
            d="M360-240q-33 0-56.5-23.5T280-320v-480q0-33 23.5-56.5T360-880h360q33 0 56.5 23.5T800-800v480q0 33-23.5 56.5T720-240H360Zm0-80h360v-480H360v480ZM200-80q-33 0-56.5-23.5T120-160v-560h80v560h440v80H200Zm160-240v-480 480Z"
          /></svg
        >
      </ThreadOptionButton>

      <ThreadOptionButton
        label={$t('thread.options.remove')}
        title={$t('thread.options.remove')}
        shortcutKey="R"
        onclick={handleRemoveThread}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.2rem"
          viewBox="0 -960 960 960"
          width="1.2rem"
          fill="currentColor"
          ><path
            d="M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z"
          /></svg
        >
      </ThreadOptionButton>
    </div>
  </div>
</div>

<style>
  .thread-options-taskbar {
    position: fixed;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.5rem 0.5rem 0 0;
    padding: 1rem;
    box-shadow: 0 5px 20px rgba(0, 0, 0, 0.15);
    z-index: 1000;
    backdrop-filter: brightness(60%);
    min-width: 35ch;
    max-width: calc(100% - 2rem);
  }

  .taskbar-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: 100%;
  }

  .thread-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border);
  }

  .thread-name {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-main);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .close-button {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted);
    padding: 0.25rem;
    border-radius: 0.25rem;
    transition: all 0.2s ease;
  }

  .close-button:hover {
    background: color-mix(in srgb, var(--error), transparent 90%);
    color: var(--error);
  }

  .taskbar-actions {
    display: flex;
    justify-content: center;
    gap: 1.5rem;
    overflow-x: auto;
    margin: 0;
  }

  @media (max-width: 768px) {
    .thread-options-taskbar {
      min-width: 100%;
      max-width: none;
    }

    .taskbar-actions {
      gap: 0.5rem;
    }
  }
</style>
