<script lang="ts">
  /**
   * Modern, minimal Modal component for managing tags.
   * Refactored for better readability and Svelte 5 idiomatic patterns.
   */
  import {
    addNoteTag,
    getTagSuggestions,
    inputManager,
    removeNoteTag,
    sessionState,
    t,
  } from '$lib';
  import { tagSuggestionShortcuts } from '$lib/config/shortcuts';

  let newTag = $state('');
  let suggestedTags = $state<string[]>([]);
  let selectedIndex = $state(-1);

  let currentTags = $derived(
    sessionState.todayNoteContent?.metadata.tags || [],
  );
  let navigationTags = $derived([...currentTags, ...suggestedTags]);

  /**
   * Fetches suggestions from the backend.
   */
  const updateSuggestions = async () => {
    suggestedTags = await getTagSuggestions(newTag);
  };

  /**
   * Toggles a tag on or off for the current note.
   */
  const handleToggleTag = async (tagToToggle?: string) => {
    const tag = (tagToToggle || newTag).trim();
    if (!tag) return;

    const isRemoving = currentTags.includes(tag);
    const updatedContent = isRemoving
      ? await removeNoteTag(tag)
      : await addNoteTag(tag);

    if (updatedContent) {
      sessionState.todayNoteContent = updatedContent;
      updateSuggestions();
    }

    newTag = '';
    selectedIndex = -1;
  };

  /**
   * Moves the keyboard selection index up or down.
   */
  const moveSelection = (direction: 'up' | 'down') => {
    const count = navigationTags.length;
    if (count === 0) return;
    selectedIndex =
      direction === 'down'
        ? (selectedIndex + 1) % count
        : (selectedIndex - 1 + count) % count;
  };

  /**
   * Selects the currently highlighted tag suggestion.
   */
  const selectCurrentSuggestion = () => {
    const tag = selectedIndex >= 0 ? navigationTags[selectedIndex] : undefined;
    handleToggleTag(tag);
  };

  /**
   * Removes the last tag from the note if the input field is empty.
   */
  const removeLastActiveTag = () => {
    if (!newTag && currentTags.length > 0) {
      handleToggleTag(currentTags[currentTags.length - 1]);
    }
  };

  /**
   * Handles keyboard events for navigation and actions.
   */
  const handleKeyDown = (e: KeyboardEvent) => {
    const isSecondary = sessionState.isMac ? e.altKey : e.metaKey || e.altKey;

    if (isSecondary && !e.shiftKey && !e.ctrlKey) {
      const shortcutIndex = tagSuggestionShortcuts.codes.indexOf(e.code);

      if (shortcutIndex !== -1 && shortcutIndex < navigationTags.length) {
        e.preventDefault();
        handleToggleTag(navigationTags[shortcutIndex]);
        return;
      }
    }

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        moveSelection('down');
        break;
      case 'ArrowUp':
        e.preventDefault();
        moveSelection('up');
        break;
      case 'Enter':
        selectCurrentSuggestion();
        break;
      case 'Backspace':
        removeLastActiveTag();
        break;
    }
  };

  $effect(() => {
    updateSuggestions();
  });
</script>

{#snippet tagItem(tag: string)}
  {@const isAdded = currentTags.includes(tag)}
  {@const globalIndex = navigationTags.indexOf(tag)}
  {@const shortcutLabel = tagSuggestionShortcuts.labels[globalIndex]}
  <button
    class="suggestion-item"
    class:selected={globalIndex === selectedIndex}
    class:is-added={isAdded}
    onclick={() => handleToggleTag(tag)}
  >
    <span class="hashtag">#</span>
    <span class="tag-label">{tag}</span>

    {#if shortcutLabel}
      <span class="shortcut-hint">
        <span class="mod">{inputManager.secondaryLabel}</span>
        <span class="key">{shortcutLabel}</span>
      </span>
    {/if}

    {#if isAdded}
      <span class="status-badge">{$t('tag.remove')}</span>
    {/if}
  </button>
{/snippet}

<div class="tag-manager">
  <div class="search-section">
    <div class="input-container">
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        bind:value={newTag}
        onkeydown={handleKeyDown}
        oninput={() => (selectedIndex = -1)}
        placeholder={$t('tag.placeholder')}
        spellcheck="false"
        autofocus
      />
    </div>

    <div class="suggestions-container">
      {#if currentTags.length > 0}
        <div class="section">
          <div class="section-label">{$t('tag.tags')}</div>
          <div class="items">
            {#each currentTags as tag}
              {@render tagItem(tag)}
            {/each}
          </div>
        </div>
      {/if}

      {#if suggestedTags.length > 0}
        <div class="section">
          <div class="section-label">{$t('tag.suggestions')}</div>
          <div class="items">
            {#each suggestedTags as tag}
              {@render tagItem(tag)}
            {/each}
          </div>
        </div>
      {:else if newTag}
        <div class="no-results">
          <p>Press Enter to create new tag <strong>#{newTag}</strong></p>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .tag-manager {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .search-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .input-container {
    position: relative;
    display: flex;
    align-items: center;
  }

  input {
    width: 100%;
    background-color: var(--bg-surface);
    border: 1px solid var(--border);
    color: var(--text-main);
    padding: 0.75rem 1rem;
    font-size: 1rem;
    outline: none;
    transition: all 0.15s ease;
    border-radius: 6px;
  }

  input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(var(--accent-rgb), 0.2);
  }

  .suggestions-container {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .section-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05rem;
    padding: 0 0.5rem;
  }

  .items {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem 0.75rem;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-main);
    cursor: pointer;
    font-size: 0.95rem;
    transition: all 0.15s ease;
    border-radius: 4px;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background-color: var(--accent);
    color: var(--accent-text);
  }

  .suggestion-item.is-added:hover,
  .suggestion-item.is-added.selected {
    background-color: var(--remove);
    color: var(--accent-text);
  }

  .tag-label {
    flex: 1;
    font-weight: 400;
  }

  .hashtag {
    color: var(--text-muted);
    font-weight: 400;
    opacity: 0.6;
  }

  .shortcut-hint {
    font-family: var(--font-mono, monospace);
    font-size: 0.75rem;
    color: var(--text-muted);
    border: 1px solid var(--border);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 0.2rem;
    opacity: 0.6;
    transition: all 0.15s ease;
  }

  .mod {
    font-size: 0.65rem;
    opacity: 0.7;
  }

  .key {
    font-weight: 600;
    text-transform: uppercase;
  }

  .status-badge {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.7rem;
    font-weight: 500;
    text-transform: uppercase;
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
    color: var(--text-muted);
    transition: all 0.15s ease;
  }

  .suggestion-item.is-added:hover .status-badge,
  .suggestion-item.is-added.selected .status-badge {
    color: var(--accent-text);
  }

  .no-results {
    padding: 1.5rem;
    text-align: center;
    color: var(--text-muted);
    background-color: var(--bg-surface);
    border-radius: 6px;
    border: 1px dashed var(--border);
    font-size: 0.9rem;
  }

  .no-results strong {
    color: var(--accent);
  }
</style>
