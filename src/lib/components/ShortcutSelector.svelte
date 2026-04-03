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
    {$t('settings.shortcuts.button')}
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
    padding: 0.65rem 1.25rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
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
