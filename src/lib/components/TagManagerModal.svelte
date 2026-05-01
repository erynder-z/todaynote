<script lang="ts">
  /**
   * Modern, minimal Modal component for managing tags.
   * Refactored for better readability and Svelte 5 idiomatic patterns.
   */
  import {
    addNoteTag,
    getTagSuggestions,
    inputManager,
    ListNavigator,
    removeNoteTag,
    sessionState,
    t,
  } from '$lib';
  import { tagSuggestionShortcuts } from '$lib/config/shortcuts';

  let newTag = $state('');
  let suggestedTags = $state<string[]>([]);

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

    const content = sessionState.todayNoteContent?.content || '';
    const isRemoving = currentTags.includes(tag);

    const updatedContent = isRemoving
      ? await removeNoteTag(tag, content)
      : await addNoteTag(tag, content);

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
    if (!newTag && currentTags.length > 0) {
      handleToggleTag(currentTags[currentTags.length - 1]);
    }
  };

  const nav = new ListNavigator(
    () => navigationTags.length,
    (i) => handleToggleTag(navigationTags[i]),
  );

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

    // Try handling list navigation first
    if (nav.handleKey(e)) return;

    // Handle Enter specifically for adding the typed tag
    if (e.key === 'Enter') {
      handleToggleTag();
      return;
    }

    // Handle backspace specifically for tag removal
    if (e.key === 'Backspace') {
      removeLastActiveTag();
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
    class:selected={globalIndex === nav.index}
    class:is-added={isAdded}
    onclick={() => handleToggleTag(tag)}
    onmouseenter={() => (nav.index = globalIndex)}
  >
    <span class="hashtag">#</span>
    <span class="tag-label">{tag}</span>

    {#if shortcutLabel}
      <span class="shortcut-hint">
        <span class="mod">{inputManager.primaryLabel}</span>
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
        oninput={() => (nav.index = -1)}
        placeholder={$t('tag.placeholder')}
        spellcheck="false"
        autofocus
      />
    </div>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="suggestions-container" onmouseleave={() => (nav.index = -1)}>
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
    background: color-mix(in srgb, var(--accent), transparent 80%);
    border: none;
    color: var(--text-main);
    padding: 0.75rem 1rem;
    font-size: 1rem;
    outline: none;
    transition: all 0.15s ease;
    border-radius: 0.5rem;
  }

  input:focus {
    border-color: var(--accent);
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

  .suggestion-item.selected {
    background-color: var(--accent);
    color: var(--accent-text);
  }

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

  .suggestion-item.selected .hashtag,
  .suggestion-item.is-added.selected .hashtag {
    color: var(--accent-text);
    opacity: 0.9;
  }

  .shortcut-hint {
    font-family: var(--font-mono, monospace);
    font-size: 0.75rem;
    color: var(--text-muted);
    border: 1px solid var(--border);
    padding: 0.125rem 0.375rem;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.2rem;
    opacity: 0.6;
  }

  .suggestion-item.selected .shortcut-hint,
  .suggestion-item.is-added.selected .shortcut-hint {
    color: var(--accent-text);
    opacity: 0.9;
    border-color: color-mix(in srgb, var(--accent-text), transparent 30%);
  }

  .mod {
    font-size: 0.65rem;
    opacity: 0.7;
  }

  .suggestion-item.selected .shortcut-hint .mod,
  .suggestion-item.is-added.selected .shortcut-hint .mod {
    opacity: 0.9;
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
    border-radius: 0.5rem;
    color: var(--text-muted);
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
    border-radius: 0.5rem;
    border: 1px dashed var(--border);
    font-size: 0.9rem;
  }

  .no-results strong {
    color: var(--accent);
  }
</style>
