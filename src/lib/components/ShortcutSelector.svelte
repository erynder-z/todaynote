<script lang="ts">
  /**
   * Component that provides a button to open the keyboard shortcuts modal.
   */
  import { sessionState, t } from '$lib';

  const openShortcuts = () => {
    sessionState.activePopup = 'shortcuts';
  };
</script>

<div class="shortcut-selector">
  <button onclick={openShortcuts} class="btn-primary">
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1.25rem"
      viewBox="0 -960 960 960"
      width="1.25rem"
      fill="currentColor"
      ><path
        d="M120-160q-33 0-56.5-23.5T40-240v-480q0-33 23.5-56.5T120-800h720q33 0 56.5 23.5T920-720v480q0 33-23.5 56.5T840-160H120Zm0-80h720v-480H120v480Zm200-40h320v-80H320v80ZM200-420h80v-80h-80v80Zm160 0h80v-80h-80v80Zm160 0h80v-80h-80v80Zm160 0h80v-80h-80v80ZM200-560h80v-80h-80v80Zm160 0h80v-80h-80v80Zm160 0h80v-80h-80v80Zm160 0h80v-80h-80v80ZM120-240v-480 480Z"
      /></svg
    >{$t('settings.shortcuts.button')}
  </button>
</div>

<style>
  .shortcut-selector {
    width: 100%;
    display: flex;
    justify-content: center;
    padding-top: 1rem;
  }

  .btn-primary {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.4rem;
    padding: 0.5rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      opacity 0.15s cubic-bezier(0.2, 0, 0, 1),
      transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
    color: var(--accent-text);
    background-color: var(--accent);
    min-width: 15ch;
  }

  .btn-primary:hover {
    opacity: 0.9;
    transform: translateY(-1px);
  }

  .btn-primary:active {
    transform: translateY(0);
  }
</style>
