<script lang="ts">
  import { sessionState } from '$lib/stores/sessionState.svelte';
  import { t } from '$lib/utils/i18n';
  import { useShortcuts } from '$lib/utils/shortcuts';

  /**
   * Toggle to switch between navigation and action mode
   */

  const toggleThreadMode = () => {
    sessionState.threadShortcutsMode =
      sessionState.threadShortcutsMode === 'navigation'
        ? 'actions'
        : 'navigation';
  };

  useShortcuts({
    toggleThreadOptionsMode: toggleThreadMode,
  });
</script>

<button
  class="mode-toggle-btn"
  data-mode={sessionState.threadShortcutsMode}
  onclick={toggleThreadMode}
  title={$t(
    sessionState.threadShortcutsMode === 'navigation'
      ? 'sidebar.thread_mode_actions'
      : 'sidebar.thread_mode_navigation',
  )}
>
  <span class="mode-label">
    {$t(
      sessionState.threadShortcutsMode === 'navigation'
        ? 'sidebar.thread_mode_navigation_short'
        : 'sidebar.thread_mode_actions_short',
    )}
  </span>
</button>

<style>
  .mode-toggle-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    padding: 0.1rem 0.5rem;
    cursor: pointer;
    font-size: 0.6rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-ui-muted);
    border-radius: 0.25rem;
    transition: all 0.2s ease;
    height: 1.2rem;
    font-weight: 600;
  }

  .mode-toggle-btn:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 90%);
    color: var(--accent);
  }

  .mode-label {
    display: inline-block;
  }

  .mode-toggle-btn[data-mode='navigation'] .mode-label {
    color: var(--text-main);
  }

  .mode-toggle-btn[data-mode='actions'] .mode-label {
    color: var(--text-main);
  }
</style>
