<script lang="ts">
  /**
   * FindInView component providing find-in-page browser-like functionality.
   * Toggleable via the "primary + F" keyboard shortcut.
   */
  import { tick } from 'svelte';
  import { fly } from 'svelte/transition';
  import { sessionState } from '../stores/sessionState.svelte';
  import { useShortcuts } from '../utils/shortcuts';

  let inputEl = $state<HTMLInputElement | null>(null);
  let searchQuery = $state('');

  // Close the search in view pane
  const close = () => {
    sessionState.showFindInView = false;
  };

  // Focus and select the input text whenever the component is shown
  $effect(() => {
    if (sessionState.showFindInView) {
      tick().then(() => {
        if (inputEl) {
          inputEl.focus();
          inputEl.select();
        }
      });
    }
  });

  // Local keydown handler in the input field to capture Escape, Enter, etc.
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      close();
      e.preventDefault();
      e.stopPropagation();
    }
  };

  // Register the global closePopup shortcut (typically Escape) to intercept and close this
  // panel before closing any background modal.
  useShortcuts({
    closePopup: () => {
      close();
    },
  });
</script>

{#if sessionState.showFindInView}
  <div class="search-in-view" transition:fly={{ y: -15, duration: 150 }}>
    <div class="search-icon">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="1.2rem"
        viewBox="0 -960 960 960"
        width="1.2rem"
        fill="currentColor"
      >
        <path
          d="M784-120 532-372q-30 24-69 38t-83 14q-109 0-184.5-75.5T120-580q0-109 75.5-184.5T380-840q109 0 184.5 75.5T640-580q0 44-14 83t-38 69l252 252-56 56ZM380-400q75 0 127.5-52.5T560-580q0-75-52.5-127.5T380-760q-75 0-127.5 52.5T200-580q0 75 52.5 127.5T380-400Z"
        />
      </svg>
    </div>

    <input
      bind:this={inputEl}
      type="text"
      placeholder="Find in page..."
      bind:value={searchQuery}
      onkeydown={handleKeyDown}
      class="search-input"
    />

    <div class="search-status">0 of 0</div>

    <div class="search-actions">
      <button
        class="action-btn"
        title="Previous Match"
        aria-label="Previous Match"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
        >
          <path d="m296-360-56-56 240-240 240 240-56 56-184-184-184 184Z" />
        </svg>
      </button>

      <button class="action-btn" title="Next Match" aria-label="Next Match">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
        >
          <path d="M480-344 240-584l56-56 184 184 184-184 56 56-240 240Z" />
        </svg>
      </button>

      <div class="divider"></div>

      <button
        class="close-btn"
        onclick={close}
        title="Close Search"
        aria-label="Close Search"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
        >
          <path
            d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
          />
        </svg>
      </button>
    </div>
  </div>
{/if}

<style>
  .search-in-view {
    position: fixed;
    bottom: 1rem;
    left: 3.5rem;
    z-index: 3000;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background-color: color-mix(in srgb, var(--bg-surface), transparent 15%);
    backdrop-filter: blur(0.5rem);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0.375rem 0.625rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
    min-width: 20rem;
  }

  .search-icon {
    color: var(--text-muted);
    display: flex;
    align-items: center;
    padding-left: 0.25rem;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-main);
    font-size: 0.9rem;
    font-family: inherit;
    outline: none;
    padding: 0.25rem 0;
    min-width: 8rem;
  }

  .search-status {
    font-size: 0.8rem;
    color: var(--text-muted);
    user-select: none;
    padding: 0 0.5rem;
    white-space: nowrap;
  }

  .search-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .action-btn,
  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 0.25rem;
    border-radius: 0.25rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .action-btn:hover {
    color: var(--text-main);
    background-color: color-mix(in srgb, var(--text-muted), transparent 90%);
  }

  .close-btn:hover {
    color: var(--error, #ef4444);
    background-color: color-mix(
      in srgb,
      var(--error, #ef4444),
      transparent 90%
    );
  }

  .divider {
    width: 1px;
    height: 1rem;
    background-color: var(--border);
    margin: 0 0.25rem;
  }
</style>
