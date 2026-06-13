<script lang="ts">
  /**
   * Slide-in menu for displaying thread options
   */
  import { slide } from 'svelte/transition';
  import type { NoteThread } from '$lib/interfaces/notes';
  import { t } from '$lib/utils/i18n';
  import { sessionState } from '../stores/sessionState.svelte';
  import { useShortcuts } from '../utils/shortcuts';
  import KeyboardShortcut from './KeyboardShortcut.svelte';

  let { thread } = $props<{
    thread: NoteThread;
  }>();

  /**
   * Closes the menu
   */
  const closeMenu = () => {
    sessionState.activePopup = null;
  };

  /**
   * Deletes the current thread
   */
  const handleDeleteThread = () => {
    // TODO: Delete thread
    console.log('Delete thread:', thread.name);
    closeMenu();
  };

  useShortcuts({
    threadOptionsDelete: handleDeleteThread,
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
      <button
        class="action-button"
        title={$t('thread.options.delete')}
        onclick={handleDeleteThread}
      >
        <span>{$t('thread.options.delete')}</span>
        <div class="shortcut-hint">
          <KeyboardShortcut primary secondary key="D" />
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
    border-radius: 1rem;
    padding: 1.5rem;
    box-shadow: 0 5px 20px rgba(0, 0, 0, 0.15);
    z-index: 2000;
    transition:
      bottom 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    backdrop-filter: blur(0.5rem);
    min-width: 70ch;
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
    gap: 1.5rem;
    overflow-x: auto;
    padding: 1.5rem;
    margin: 0;
  }

  .taskbar-actions::-webkit-scrollbar {
    height: 6px;
  }

  .taskbar-actions::-webkit-scrollbar-track {
    background: transparent;
  }

  .taskbar-actions::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }

  .action-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1rem;
    background-color: color-mix(in srgb, var(--bg-surface), transparent 45%);
    border: none;
    border-radius: 0.5rem;
    color: var(--text-main);
    cursor: pointer;
    width: 100%;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
    overflow: hidden;
    min-width: 80px;
    flex-shrink: 0;
    position: relative;
  }

  .action-button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .action-button:hover:not(:disabled) {
    background-color: color-mix(in srgb, var(--accent), transparent 80%);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .action-button:focus {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .action-button span {
    font-size: 0.9rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
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
