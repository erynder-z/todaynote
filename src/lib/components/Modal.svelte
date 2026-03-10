<script lang="ts">
  /**
   * Wrapper that handles displaying a modal popup as a child component.
   */
  import { sessionState } from '$lib';

  let { title, children } = $props();

  /**
   * Clears the active popup state to hide the modal.
   */
  const close = () => (sessionState.activePopup = null);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={close}>
  <div class="popup" onclick={(e) => e.stopPropagation()}>
    <div class="popup-header">
      {#if title}<h2>{title}</h2>{/if}
      <button onclick={close} class="close-button">×</button>
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
</style>
