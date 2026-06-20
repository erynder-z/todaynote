<script lang="ts">
  /**
   * Slide-in menu for displaying thread options
   */
  import { onMount } from 'svelte';
  import { slide } from 'svelte/transition';
  import type { NoteThread } from '$lib/interfaces/notes';
  import { toast } from '$lib/stores/toast.svelte';
  import { t } from '$lib/utils/i18n';
  import { sessionState } from '../stores/sessionState.svelte';
  import { aggregateThread, removeThread } from '../utils/notes';
  import { useShortcuts } from '../utils/shortcuts';
  import KeyboardShortcut from './KeyboardShortcut.svelte';

  let { thread } = $props<{
    thread: NoteThread;
  }>();

  let aggregatedThread = $state<Awaited<
    ReturnType<typeof aggregateThread>
  > | null>(null);

  let hasLinkedThreads = $state(false);

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
      aggregatedThread = await aggregateThread(thread.name);
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
   * Deletes the current thread
   */
  const handleRemoveThread = async () => {
    try {
      const currentContent = sessionState.todayNoteContent?.content;

      if (!currentContent) {
        toast.error($t('thread.options.remove_error'));
        return;
      }

      const result = await removeThread(thread.name, currentContent);

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

  onMount(() => {
    loadLinkedThreads();
  });

  useShortcuts({
    threadOptionRemove: handleRemoveThread,
    threadOptionLinked: handleLinked,
    closePopup: closeMenu,
  });
</script>

<div
  class="thread-options-taskbar"
  class:open={sessionState.activePopup === 'threadOptions'}
  transition:slide
>
  <div class="taskbar-content">
    <div class="thread-info">
      <h3 class="thread-name">{thread.name}</h3>
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
      {#if hasLinkedThreads}
        <button
          class="action-button"
          title={$t('thread.options.linked')}
          onclick={handleLinked}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="1.5rem"
            viewBox="0 -960 960 960"
            width="1.5rem"
            fill="currentColor"
            ><path
              d="M760-600q-57 0-99-34t-56-86H354q-11 42-41.5 72.5T240-606v251q52 14 86 56t34 99q0 66-47 113T200-40q-66 0-113-47T40-200q0-57 34-99t86-56v-251q-52-14-86-56t-34-98q0-66 47-113t113-47q56 0 98 34t56 86h251q14-52 56-86t99-34q66 0 113 47t47 113q0 66-47 113t-113 47ZM200-120q33 0 56.5-24t23.5-56q0-33-23.5-56.5T200-280q-32 0-56 23.5T120-200q0 32 24 56t56 24Zm0-560q33 0 56.5-23.5T280-760q0-33-23.5-56.5T200-840q-32 0-56 23.5T120-760q0 33 24 56.5t56 23.5ZM760-40q-66 0-113-47t-47-113q0-66 47-113t113-47q66 0 113 47t47 113q0 66-47 113T760-40Zm0-80q33 0 56.5-24t23.5-56q0-33-23.5-56.5T760-280q-33 0-56.5 23.5T680-200q0 32 23.5 56t56.5 24Zm0-560q33 0 56.5-23.5T840-760q0-33-23.5-56.5T760-840q-33 0-56.5 23.5T680-760q0 33 23.5 56.5T760-680ZM200-200Zm0-560Zm560 560Zm0-560Z"
            /></svg
          >
          <span>{$t('thread.options.linked')}</span>
          <div class="shortcut-hint">
            <KeyboardShortcut primary secondary key="I" />
          </div>
        </button>
      {:else}
        <div class="action-button action-button-empty">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="1.5rem"
            viewBox="0 -960 960 960"
            width="1.5rem"
            fill="currentColor"
            ><path
              d="M760-600q-57 0-99-34t-56-86H354q-11 42-41.5 72.5T240-606v251q52 14 86 56t34 99q0 66-47 113T200-40q-66 0-113-47T40-200q0-57 34-99t86-56v-251q-52-14-86-56t-34-98q0-66 47-113t113-47q56 0 98 34t56 86h251q14-52 56-86t99-34q66 0 113 47t47 113q0 66-47 113t-113 47ZM200-120q33 0 56.5-24t23.5-56q0-33-23.5-56.5T200-280q-32 0-56 23.5T120-200q0 32 24 56t56 24Zm0-560q33 0 56.5-23.5T280-760q0-33-23.5-56.5T200-840q-32 0-56 23.5T120-760q0 33 24 56.5t56 23.5ZM760-40q-66 0-113-47t-47-113q0-66 47-113t113-47q66 0 113 47t47 113q0 66-47 113T760-40Zm0-80q33 0 56.5-24t23.5-56q0-33-23.5-56.5T760-280q-33 0-56.5 23.5T680-200q0 32 23.5 56t56.5 24Zm0-560q33 0 56.5-23.5T840-760q0-33-23.5-56.5T760-840q-33 0-56.5 23.5T680-760q0 33 23.5 56.5T760-680ZM200-200Zm0-560Zm560 560Zm0-560Z"
            /></svg
          >
          <span>{$t('thread.options.no_linked')}</span>
        </div>
      {/if}

      <button
        class="action-button"
        title={$t('thread.options.remove')}
        onclick={handleRemoveThread}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.5rem"
          viewBox="0 -960 960 960"
          width="1.5rem"
          fill="currentColor"
          ><path
            d="M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z"
          /></svg
        >
        <span>{$t('thread.options.remove')}</span>
        <div class="shortcut-hint">
          <KeyboardShortcut primary secondary key="R" />
        </div>
      </button>
    </div>
  </div>
</div>

<style>
  .thread-options-taskbar {
    position: fixed;
    bottom: -100%;
    left: 50%;
    transform: translateX(-50%);
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 0.5rem 0.5rem 0 0;
    padding: 1rem;
    box-shadow: 0 5px 20px rgba(0, 0, 0, 0.15);
    z-index: 999;
    transition:
      bottom 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    backdrop-filter: blur(0.5rem);
    min-width: 35ch;
    max-width: calc(100% - 2rem);
  }

  .thread-options-taskbar.open {
    bottom: 0.25rem;
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

  .action-button,
  .action-button-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1rem;
    background-color: color-mix(in srgb, var(--accent), transparent 80%);
    border: none;
    border-radius: 0.25rem;
    color: var(--text-main);
    cursor: pointer;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
    overflow: hidden;
    flex: 1 1 0;
    min-width: 0;
    position: relative;
  }

  .action-button-empty {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .action-button:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .action-button span {
    font-size: 0.8rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    text-align: center;
  }

  .shortcut-hint {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    background-color: var(--bg-main);
    padding: 0.1rem 0.3rem;
    border-radius: 0.15rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  @media (max-width: 768px) {
    .thread-options-taskbar {
      min-width: 100%;
      max-width: none;
    }

    .thread-options-taskbar.open {
      bottom: 0;
    }

    .taskbar-actions {
      gap: 0.5rem;
    }

    .action-button {
      padding: 0.75rem;
      min-width: 60px;
    }

    .action-button span {
      font-size: 0.8rem;
    }
  }
</style>
