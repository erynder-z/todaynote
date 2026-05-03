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
    ModalFooter,
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

<div class="tag-manager-container">
  <header class="tag-header">
    <div class="input-wrapper">
      <div class="tag-icon">
        <svg
          viewBox="0 0 24 24"
          width="18"
          height="18"
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
  </header>

  <main class="results-area" onmouseleave={() => (nav.index = -1)}>
    {#if currentTags.length === 0 && suggestedTags.length === 0 && !newTag}
      <div class="status-view empty">
        <p class="muted">{$t('tag.suggestions')}</p>
      </div>
    {:else}
      <div class="results-list">
        {#if currentTags.length > 0}
          <div class="section">
            <div class="section-label">{$t('tag.tags')}</div>
            {#each currentTags as tag}
              {@render tagItem(tag)}
            {/each}
          </div>
        {/if}

        {#if suggestedTags.length > 0}
          <div class="section">
            <div class="section-label">{$t('tag.suggestions')}</div>
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
      { label: $t('search.footer.navigate'), key: '↑↓' },
      { label: $t('shortcuts.tags.toggle'), key: 'Enter' },
      { label: $t('search.footer.close'), key: 'Esc' },
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
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-surface);
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background-color: var(--bg-main);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 0 0.75rem;
    transition:
      border-color 0.2s,
      box-shadow 0.2s;
  }

  .input-wrapper:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent), transparent 80%);
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

  .section {
    display: flex;
    flex-direction: column;
  }

  .section-label {
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05rem;
    padding: 0.75rem 1rem 0.25rem 1rem;
    background-color: var(--bg-surface);
    border-bottom: 1px solid var(--border);
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.75rem 1rem;
    text-align: left;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-main);
    cursor: pointer;
    font-size: 0.95rem;
    transition: background-color 0.1s;
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

  .tag-label {
    flex: 1;
    font-weight: 500;
  }

  .hashtag {
    color: var(--accent);
    font-weight: 600;
  }

  .suggestion-item.selected .tag-label {
    color: var(--accent);
  }

  .shortcut-hint {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    background-color: var(--bg-main);
    border: 1px solid var(--border);
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .suggestion-item.selected .shortcut-hint {
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
