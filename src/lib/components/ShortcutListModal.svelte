<script lang="ts">
  /**
   * Modal that displays all available keyboard shortcuts.
   */
  import { KeyboardShortcut, t } from '$lib';
  import { defaultShortcuts } from '$lib/config/shortcuts';

  // Global shortcuts (exclude context-specific ones)
  const globalActions = [
    'toggleSearch',
    'toggleNoteBrowser',
    'toggleSettings',
    'manageTags',
    'closePopup',
    'focusLastLine',
    'jumpByNumber',
    'navigateYesterday',
    'navigateLastAvailable',
    'navigateToday',
  ];
  const shortcuts = globalActions.map(
    (action) => defaultShortcuts[action as keyof typeof defaultShortcuts],
  );
  const fuzzyShortcut = defaultShortcuts.toggleFuzzy;
  const searchModeShortcut = defaultShortcuts.toggleSearchMode;
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
            <KeyboardShortcut
              primary={shortcut.primary}
              secondary={shortcut.secondary}
              key={shortcut.key}
            />
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
        <KeyboardShortcut primary secondary key="1-9, A-Z" />
      </div>
    </div>
  </div>

  {#if fuzzyShortcut || searchModeShortcut}
    <div class="search-shortcuts">
      <h3>{$t('shortcuts.search.description')}</h3>
      {#if fuzzyShortcut}
        <div class="shortcut-item">
          <span class="shortcut-description"
            >{$t('shortcuts.search.toggle_fuzzy')}</span
          >
          <div class="shortcut-keys">
            <KeyboardShortcut
              primary={fuzzyShortcut.primary}
              secondary={fuzzyShortcut.secondary}
              key={fuzzyShortcut.key.toUpperCase()}
            />
          </div>
        </div>
      {/if}
      {#if searchModeShortcut}
        <div class="shortcut-item">
          <span class="shortcut-description"
            >{$t('shortcuts.search.toggle_mode')}</span
          >
          <div class="shortcut-keys">
            <KeyboardShortcut
              primary={searchModeShortcut.primary}
              secondary={searchModeShortcut.secondary}
              key={searchModeShortcut.key.toUpperCase()}
            />
          </div>
        </div>
      {/if}
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

  .tag-shortcuts h3,
  .search-shortcuts h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    color: var(--accent);
  }
</style>
