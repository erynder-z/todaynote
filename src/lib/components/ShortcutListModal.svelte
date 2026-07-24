<script lang="ts">
  /**
   * Modal that displays all available keyboard shortcuts.
   */
  import { KeyboardShortcut, t } from '$lib';
  import { settings } from '$lib/stores/settings.svelte';
  import type { ShortcutAction } from '$lib/types/input';

  // Global shortcuts (exclude context-specific ones)
  const globalActions: ShortcutAction[] = [
    'toggleSearch',
    'toggleFindInNote',
    'toggleNoteBrowser',
    'toggleSettings',
    'toggleStatistics',
    'toggleSidebar',
    'manageTags',
    'closePopup',
    'focusLastLine',
    'jumpByNumber',
    'navigateYesterday',
    'navigateLastAvailable',
    'navigateToday',
    'toggleThreadOptionsMode',
  ];
  const shortcuts = globalActions.map((action) => settings.shortcuts[action]);
  const fuzzyShortcut = settings.shortcuts.toggleFuzzy;
  const searchModeShortcut = settings.shortcuts.toggleSearchMode;
  const threadOptionRemoveShortcut = settings.shortcuts.threadOptionRemove;
  const threadOptionLinkedShortcut = settings.shortcuts.threadOptionLinked;

  // Editor formatting shortcuts
  const toggleBoldShortcut = settings.shortcuts.toggleBold;
  const toggleItalicShortcut = settings.shortcuts.toggleItalic;
  const toggleStrikethroughShortcut = settings.shortcuts.toggleStrikethrough;
  const toggleCodeShortcut = settings.shortcuts.toggleCode;
  const toggleBlockquoteShortcut = settings.shortcuts.toggleBlockquote;
  const toggleLinkShortcut = settings.shortcuts.toggleLink;
  const copySelectionShortcut = settings.shortcuts.copySelection;
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

  <div class="thread-option-shortcuts">
    <h3>{$t('shortcuts.thread.description')}</h3>
    {#if threadOptionRemoveShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description">{$t('shortcuts.thread.delete')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={threadOptionRemoveShortcut.primary}
            secondary={threadOptionRemoveShortcut.secondary}
            key={threadOptionRemoveShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
    {#if threadOptionLinkedShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description">{$t('shortcuts.thread.linked')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={threadOptionLinkedShortcut.primary}
            secondary={threadOptionLinkedShortcut.secondary}
            key={threadOptionLinkedShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
  </div>

  <div class="editor-formatting-shortcuts">
    <h3>{$t('shortcuts.editor_formatting.description')}</h3>
    {#if toggleBoldShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description"
          >{$t('shortcuts.editor_formatting.bold')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={toggleBoldShortcut.primary}
            secondary={toggleBoldShortcut.secondary}
            key={toggleBoldShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
    {#if toggleItalicShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description"
          >{$t('shortcuts.editor_formatting.italic')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={toggleItalicShortcut.primary}
            secondary={toggleItalicShortcut.secondary}
            key={toggleItalicShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
    {#if toggleStrikethroughShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description"
          >{$t('shortcuts.editor_formatting.strikethrough')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={toggleStrikethroughShortcut.primary}
            secondary={toggleStrikethroughShortcut.secondary}
            key={toggleStrikethroughShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
    {#if toggleCodeShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description"
          >{$t('shortcuts.editor_formatting.code')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={toggleCodeShortcut.primary}
            secondary={toggleCodeShortcut.secondary}
            key={toggleCodeShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
    {#if toggleBlockquoteShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description"
          >{$t('shortcuts.editor_formatting.blockquote')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={toggleBlockquoteShortcut.primary}
            secondary={toggleBlockquoteShortcut.secondary}
            key={toggleBlockquoteShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
    {#if toggleLinkShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description"
          >{$t('shortcuts.editor_formatting.link')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={toggleLinkShortcut.primary}
            secondary={toggleLinkShortcut.secondary}
            key={toggleLinkShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
    {#if copySelectionShortcut}
      <div class="shortcut-item">
        <span class="shortcut-description"
          >{$t('shortcuts.editor_formatting.copy')}</span
        >
        <div class="shortcut-keys">
          <KeyboardShortcut
            primary={copySelectionShortcut.primary}
            secondary={copySelectionShortcut.secondary}
            key={copySelectionShortcut.key.toUpperCase()}
          />
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .shortcut-list {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    padding: 2rem;
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
  .search-shortcuts h3,
  .thread-option-shortcuts h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
    color: var(--accent);
  }
</style>
