<script lang="ts">
  /**
   * Modal wrapper that handles displaying a popup as a child component.
   * Consolidates focus management and scroll locking into a single setup action.
   */
  import { type Component, tick } from 'svelte';
  import type { ModalSize } from '$lib/types/ui';
  import { focusTrap } from '../actions/focusTrap';
  import { sessionState } from '../stores/sessionState.svelte';
  import { settings } from '../stores/settings.svelte';
  import { useShortcuts } from '../utils/shortcuts';
  import LayoutToggle from './LayoutToggle.svelte';

  let {
    title,
    children,
    size = 'md',
    showLayoutToggle = false,
  } = $props<{
    title?: string;
    children: Component;
    size?: ModalSize;
    showLayoutToggle?: boolean;
  }>();

  /**
   * Clears the active popup state to hide the modal.
   */
  const close = () => {
    sessionState.activePopup = null;
  };

  /**
   * Updates the notes list layout in the settings.
   */
  const setLayout = (layout: 'list' | 'masonry') => {
    settings.setNotesListLayout(layout);
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

    const setFocus = async () => {
      // tick() ensures Svelte has finished updating the DOM and processed snippets
      await tick();

      // If focus is already within the modal (e.g. via browser autofocus), don't steal it
      if (
        node.contains(document.activeElement) &&
        document.activeElement !== node
      ) {
        return;
      }

      const autofocusElement = node.querySelector('[autofocus]') as HTMLElement;
      if (autofocusElement) {
        autofocusElement.focus();
      } else {
        node.focus();
      }
    };

    const restoreFocus = () => {
      if (previousFocus && typeof previousFocus.focus === 'function')
        previousFocus.focus();
    };

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
    class="popup size-{size}"
    use:setupModal
    use:focusTrap
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
  >
    <div class="popup-header">
      <div class="header-left">
        {#if showLayoutToggle}
          <LayoutToggle onLayoutChange={setLayout} />
        {/if}
      </div>
      {#if title}<h2>{title}</h2>{/if}
      <div class="header-right">
        <button onclick={close} class="close-button" aria-label="Close"
          ><svg
            xmlns="http://www.w3.org/2000/svg"
            height="1.2rem"
            viewBox="0 -960 960 960"
            width="1.2rem"
            fill="currentColor"
            ><path
              d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
            /></svg
          ></button
        >
      </div>
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
    max-width: 90dvw;
    height: 85dvh;
    color: var(--text-main);
    outline: none;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    transition: width 0.3s cubic-bezier(0.2, 0, 0, 1);
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  /* Size Variants */
  .size-sm {
    width: 25rem;
  }
  .size-md {
    width: 40rem;
  }
  .size-lg {
    width: 60rem;
  }
  .size-xl {
    width: 80rem;
  }
  .size-full {
    width: 95dvw;
    max-width: 100dvw;
    height: 95dvh;
  }

  @media (max-width: 768px) {
    .popup {
      width: 95dvw !important;
      padding: 1rem;
    }
  }

  .popup-header {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: var(--bg-surface);
    border-bottom: 1px solid var(--border);
  }

  .header-left {
    display: flex;
    justify-content: flex-start;
  }

  .header-right {
    display: flex;
    justify-content: flex-end;
  }

  h2 {
    margin: 0;
    font-size: 1.1rem;
    text-align: center;
    font-weight: 800;
    color: var(--text-main);
  }

  .close-button {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted);
    transition: color 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .close-button:hover {
    color: var(--error);
  }

  .popup-content {
    flex: 1;
    overflow-y: auto;
  }
</style>
