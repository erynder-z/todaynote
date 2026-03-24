<script lang="ts">
  /**
   * Modal wrapper that handles displaying a popup as a child component.
   * Consolidates focus management and scroll locking into a single setup action.
   */
  import { focusTrap, sessionState, useShortcuts } from '$lib';

  let { title, children } = $props();

  /**
   * Clears the active popup state to hide the modal.
   */
  const close = () => {
    sessionState.activePopup = null;
  };

  useShortcuts({
    closePopup: () => close(),
  });

  /**
   * Manages the lifecycle of the modal:
   * 1. Locks background scrolling.
   * 2. Sets initial focus to the modal.
   * 3. Restores both scroll and focus on destruction.
   */
  const setupModal = (node: HTMLElement) => {
    const previousFocus = document.activeElement as HTMLElement;
    const previousOverflow = document.body.style.overflow;

    const lockScroll = () => (document.body.style.overflow = 'hidden');
    const unlockScroll = () =>
      (document.body.style.overflow = previousOverflow);
    const setFocus = () => node.focus();
    const restoreFocus = () => previousFocus?.focus();

    lockScroll();
    setFocus();

    return {
      destroy() {
        unlockScroll();
        restoreFocus();
      },
    };
  };
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={close}>
  <div
    class="popup"
    use:setupModal
    use:focusTrap
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="popup-header">
      {#if title}<h2>{title}</h2>{/if}
      <button onclick={close} class="close-button" aria-label="Close">×</button>
    </div>
    <div class="popup-content">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 2000;
    backdrop-filter: blur(0.25rem);
  }

  .popup {
    background-color: var(--bg-base);
    padding: 2rem;
    border-radius: 1rem;
    border: 0.0625rem solid var(--border);
    max-width: 90vw;
    max-height: 85vh;
    overflow-y: auto;
    width: 31.25rem;
    color: var(--text-main);
    outline: none;
  }

  .popup-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  h2 {
    margin: 0;
    font-size: 1.4rem;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 2rem;
    cursor: pointer;
    color: var(--text-muted);
    transition: color 0.2s;
  }

  .close-button:hover {
    color: var(--error);
  }

  .popup-content {
    /* Ensure content within popup still scrolls if needed */
    max-height: calc(85vh - 6rem);
    overflow-y: auto;
  }
</style>
