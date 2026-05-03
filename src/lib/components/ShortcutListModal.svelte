<script lang="ts">
  /**
   * Modal that displays all available keyboard shortcuts.
   */
  import { inputManager, t } from '$lib';
  import { defaultShortcuts } from '$lib/config/shortcuts';

  // Global shortcuts (exclude context-specific ones)
  const globalActions = [
    'toggleSearch',
    'toggleNotesList',
    'toggleSettings',
    'manageTags',
    'closePopup',
    'focusLastLine',
    'jumpByNumber',
  ];
  const shortcuts = globalActions.map(
    (action) => defaultShortcuts[action as keyof typeof defaultShortcuts],
  );
  const fuzzyShortcut = defaultShortcuts.toggleFuzzy;
</script>

<div class="shortcut-list">
  <div class="shortcut-grid">
    {#each shortcuts as shortcut}
      {#if shortcut}
        <div class="shortcut-item">
          <span class="shortcut-description">
            {#if shortcut.description}
              {$t(
                `shortcuts.action.${shortcut.description.toLowerCase().replace(/\s+/g, '_')}`,
              )}
            {:else}
              {shortcut.key}
            {/if}
          </span>
          <div class="shortcut-keys">
            {#if shortcut.primary}
              <kbd>{inputManager.primaryLabel}</kbd>
              <span class="plus">+</span>
            {/if}
            {#if shortcut.secondary}
              <kbd>{inputManager.secondaryLabel}</kbd>
              <span class="plus">+</span>
            {/if}
            <kbd
              >{shortcut.key === ' '
                ? 'Space'
                : shortcut.key.length > 1 && shortcut.key.includes(',')
                  ? '1-9, A-K'
                  : shortcut.key}</kbd
            >
          </div>
        </div>
      {/if}
    {/each}
  </div>

  <div class="tag-shortcuts">
    <h3>{$t('shortcuts.tags.description')}</h3>
    <div class="shortcut-item">
      <span class="shortcut-description">{$t('shortcuts.tags.toggle')}</span>
      <div class="shortcut-keys">
        <kbd>{inputManager.primaryLabel}</kbd>
        <span class="plus">+</span>
        <kbd>{inputManager.secondaryLabel}</kbd>
        <span class="plus">+</span>
        <kbd>1-9, A-Z</kbd>
      </div>
    </div>
  </div>

  {#if fuzzyShortcut}
    <div class="search-shortcuts">
      <h3>{$t('shortcuts.search.description')}</h3>
      <div class="shortcut-item">
        <span class="shortcut-description"
          >{$t('shortcuts.search.toggle_fuzzy')}</span
        >
        <div class="shortcut-keys">
          {#if fuzzyShortcut.primary}
            <kbd>{inputManager.primaryLabel}</kbd>
            <span class="plus">+</span>
          {/if}
          {#if fuzzyShortcut.secondary}
            <kbd>{inputManager.secondaryLabel}</kbd>
            <span class="plus">+</span>
          {/if}
          <kbd>{fuzzyShortcut.key.toUpperCase()}</kbd>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .shortcut-list {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    color: var(--text-main);
  }

  .shortcut-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .shortcut-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border);
  }

  .shortcut-item:last-child {
    border-bottom: none;
  }

  .shortcut-description {
    font-size: 0.95rem;
    color: var(--text-muted);
  }

  .shortcut-keys {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  kbd {
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.2rem 0.5rem;
    font-family: var(--font-mono, monospace);
    font-size: 0.85rem;
    color: var(--text-main);
    box-shadow: 0 2px 0 var(--border);
    min-width: 1.5rem;
    text-align: center;
  }

  .plus {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .tag-shortcuts h3,
  .search-shortcuts h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    color: var(--accent);
  }
</style>
