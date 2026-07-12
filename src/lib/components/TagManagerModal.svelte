<script lang="ts">
  /**
   * Modern, minimal Modal component for managing tags.
   * Refactored for better readability and Svelte 5 idiomatic patterns.
   */
  import { tagSuggestionShortcuts } from '$lib/config/shortcuts';
  import { inputManager } from '../stores/input.svelte';
  import { ListNavigator } from '../stores/listNav.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import { settings } from '../stores/settings.svelte';
  import { t } from '../utils/i18n';
  import { notesService } from '../utils/notes';
  import { useShortcuts } from '../utils/shortcuts';
  import KeyboardShortcut from './KeyboardShortcut.svelte';
  import MasonryLayout from './MasonryLayout.svelte';
  import ModalFooter from './ModalFooter.svelte';

  let newTag = $state('');
  let suggestedTags = $state<string[]>([]);
  let currentTags = $derived(
    sessionState.todayNoteContent?.metadata.tags || [],
  );
  let navigationTags = $derived([...currentTags, ...suggestedTags]);

  useShortcuts({
    toggleNoteBrowserLayout: () => {
      const nextLayout =
        settings.notesListLayout === 'list' ? 'masonry' : 'list';
      settings.setNotesListLayout(nextLayout);
    },
  });

  /**
   * Fetches suggestions from the backend.
   */
  const updateSuggestions = async () => {
    suggestedTags = await notesService.getTagSuggestions(newTag);
  };

  /**
   * Toggles a tag on or off for the current note.
   */
  const handleToggleTag = async (tagToToggle?: string) => {
    const tag = (tagToToggle || newTag).trim();
    if (!tag) return;

    const content = sessionState.todayNoteContent?.content || '';
    const isRemoving = currentTags.includes(tag);

    const updatedContent = isRemoving
      ? await notesService.removeNoteTag(tag, content)
      : await notesService.addNoteTag(tag, content);

    if (updatedContent) {
      sessionState.todayNoteContent = updatedContent;
      updateSuggestions();
    }

    newTag = '';
    nav.reset();
  };

  /**
   * Removes the last tag from the note if the input field is empty.
   */
  const removeLastActiveTag = () => {
    if (!newTag && currentTags.length > 0)
      handleToggleTag(currentTags[currentTags.length - 1]);
  };

  const nav = new ListNavigator(
    () => navigationTags.length,
    (i) => handleToggleTag(navigationTags[i]),
  );

  let masonryLayout: { handleKey: (e: KeyboardEvent) => boolean } | null =
    $state(null);

  /**
   * Handles keyboard events for navigation and actions.
   */
  const handleKeyDown = (e: KeyboardEvent) => {
    inputManager.updateModifiers(e);

    const isPrimary = inputManager.primaryPressed;
    const isSecondary = inputManager.secondaryPressed;

    if (isPrimary && isSecondary) {
      const shortcutIndex = tagSuggestionShortcuts.codes.indexOf(e.code);

      if (shortcutIndex !== -1 && shortcutIndex < navigationTags.length) {
        e.preventDefault();
        e.stopPropagation();
        handleToggleTag(navigationTags[shortcutIndex]);
        return;
      }
    }

    if (settings.notesListLayout === 'masonry' && masonryLayout) {
      if (masonryLayout.handleKey(e)) return;
    }

    // Try handling list navigation first
    if (nav.handleKey(e)) return;

    // Handle Enter specifically for adding the typed tag
    if (e.key === 'Enter') {
      handleToggleTag();
      return;
    }

    // Handle backspace specifically for tag removal
    if (e.key === 'Backspace') removeLastActiveTag();
  };

  $effect(() => {
    updateSuggestions();
  });

  $effect(() => {
    if (nav.index !== -1 && nav.lastInputSource === 'keyboard') {
      const selected = document.querySelector(
        '.item-card.selected, .suggestion-item.selected',
      );
      selected?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  });
</script>

{#snippet tagSnippet(tag: string)}
  {@const isAdded = currentTags.includes(tag)}
  {@const globalIndex = navigationTags.indexOf(tag)}
  {@const shortcutLabel = tagSuggestionShortcuts.labels[globalIndex]}
  {@const isMasonry = settings.notesListLayout === 'masonry'}
  <div
    class="tag-content-wrapper"
    class:is-added={isAdded}
    class:is-masonry={isMasonry}
  >
    <span class="tag-pill">{tag}</span>

    {#if shortcutLabel}
      <span class="shortcut-hint">
        <KeyboardShortcut primary secondary key={shortcutLabel} />
      </span>
    {/if}

    {#if isAdded}
      <span class="status-badge">{$t('tag.remove')}</span>
    {/if}
  </div>
{/snippet}

{#snippet tagItem(tag: string)}
  {@const isAdded = currentTags.includes(tag)}
  {@const globalIndex = navigationTags.indexOf(tag)}
  <button
    class="suggestion-item"
    class:selected={globalIndex === nav.index}
    class:is-added={isAdded}
    onclick={() => handleToggleTag(tag)}
    onmouseenter={() => {
      if (nav.shouldIgnoreMouseEnter()) return;
      nav.setIndex(globalIndex, 'mouse');
    }}
  >
    {@render tagSnippet(tag)}
  </button>
{/snippet}

<div class="tag-manager-container">
  <div class="tag-header">
    <div class="input-wrapper">
      <div class="tag-icon">
        <svg
          viewBox="0 0 24 24"
          width="1.25rem"
          height="1.25rem"
          stroke="currentColor"
          stroke-width="2"
          fill="none"
        >
          <path
            d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"
          ></path>
          <line x1="7" y1="7" x2="7.01" y2="7"></line>
        </svg>
      </div>

      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        bind:value={newTag}
        onkeydown={handleKeyDown}
        oninput={() => (nav.index = -1)}
        placeholder={$t('tag.placeholder')}
        spellcheck="false"
        autofocus
      />
    </div>
  </div>

  <main class="results-area" onmouseleave={() => (nav.index = -1)}>
    {#if currentTags.length === 0 && suggestedTags.length === 0 && !newTag}
      <div class="status-view empty">
        <p class="muted">{$t('tag.suggestions')}</p>
      </div>
    {:else if settings.notesListLayout === 'masonry'}
      {#if navigationTags.length > 0}
        <MasonryLayout
          bind:this={masonryLayout}
          items={navigationTags}
          {nav}
          onSelect={handleToggleTag}
          itemSnippet={tagSnippet}
        />
      {:else if newTag}
        <div class="status-view">
          <p>{@html $t('tag.create_new', { tag: newTag })}</p>
        </div>
      {:else}
        <div class="status-view empty">
          <p class="muted">{$t('tag.suggestions')}</p>
        </div>
      {/if}
    {:else}
      <div class="results-list">
        {#if currentTags.length > 0}
          <div class="thread">
            <div class="thread-label">{$t('tag.tags')}</div>
            {#each currentTags as tag}
              {@render tagItem(tag)}
            {/each}
          </div>
        {/if}

        {#if suggestedTags.length > 0}
          <div class="thread">
            <div class="thread-label">{$t('tag.suggestions')}</div>
            {#each suggestedTags as tag}
              {@render tagItem(tag)}
            {/each}
          </div>
        {:else if newTag}
          <div class="status-view">
            <p>{@html $t('tag.create_new', { tag: newTag })}</p>
          </div>
        {/if}
      </div>
    {/if}
  </main>

  <ModalFooter
    shortcuts={[
      ...(settings.notesListLayout === 'masonry'
        ? [{ label: $t('browser.navigate'), key: '🠝🠟 🠜🠞' }]
        : [{ label: $t('search.footer.navigate'), key: '🠝🠟' }]),
      { label: $t('shortcuts.tags.toggle'), key: '↵' },
      {
        label: $t('shortcuts.action.toggle_note_browser_layout'),
        action: 'toggleNoteBrowserLayout',
      },
    ]}
  />
</div>

<style>
  .tag-manager-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    background-color: var(--bg-main);
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .tag-header {
    padding: 1rem;
    background-color: var(--bg-surface);
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100%;
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    height: fit-content;
    background-color: color-mix(in srgb, var(--accent), transparent 95%);
    border-radius: 0.25rem;
    padding: 0 0.75rem;
    transition:
      border-color 0.15s cubic-bezier(0.2, 0, 0, 1),
      box-shadow 0.15s cubic-bezier(0.2, 0, 0, 1);
  }

  .input-wrapper:focus-within {
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
  }

  .tag-icon {
    color: var(--text-muted);
    display: flex;
    align-items: center;
  }

  input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-main);
    padding: 0.75rem 0;
    font-size: 1rem;
    outline: none;
  }

  .results-area {
    flex: 1;
    overflow-y: auto;
    min-height: 300px;
    position: relative;
  }

  .results-list {
    display: flex;
    flex-direction: column;
  }

  .thread {
    display: flex;
    flex-direction: column;
  }

  .thread-label {
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05rem;
    padding: 0.5rem;
    background-color: var(--bg-surface);
    border-bottom: 1px solid var(--border);
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 0;
    text-align: left;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-main);
    cursor: pointer;
    transition: background-color 0.1s cubic-bezier(0.2, 0, 0, 1);
  }

  .suggestion-item:last-child {
    border-bottom: none;
  }

  .suggestion-item.selected {
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
  }

  .suggestion-item.is-added.selected {
    background-color: color-mix(in srgb, var(--remove), transparent 85%);
  }

  .tag-content-wrapper {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.75rem 1rem;
    font-size: 1.25rem;
  }

  .tag-content-wrapper.is-masonry {
    flex-direction: column;
    gap: 1rem;
    min-height: 4rem;
  }

  .tag-pill {
    font-size: 0.9rem;
    padding: 0.3rem 0.8rem;
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
    border-radius: 1rem;
    font-weight: 500;
    display: inline-block;
    white-space: nowrap;
    max-width: 90%;
    overflow: hidden;
    text-overflow: ellipsis;
    vertical-align: middle;
  }

  .shortcut-hint {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    background-color: var(--bg-main);
    padding: 0.1rem 0.3rem;
    border-radius: 0.15rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .suggestion-item.selected .shortcut-hint,
  :global(.item-card.selected) .shortcut-hint {
    border-color: var(--accent);
    color: var(--accent);
  }

  .status-badge {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--remove);
    opacity: 0.8;
  }

  .status-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1rem;
    color: var(--text-muted);
    text-align: center;
  }

  .status-view p {
    max-width: 100%;
    white-space: normal;
    word-break: break-word;
    margin: 0;
  }

  :global(.status-view kbd) {
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    font-family: var(--font-mono);
  }

  :global(.status-view strong) {
    color: var(--accent);
  }

  .muted {
    font-style: italic;
  }
</style>
