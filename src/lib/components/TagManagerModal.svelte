<script lang="ts">
  /**
   * Modern, minimal Modal component for managing tags.
   * Refactored for better readability and Svelte 5 idiomatic patterns.
   */
  import { onMount } from 'svelte';
  import {
    addNoteTag,
    getAllTags,
    inputManager,
    removeNoteTag,
    sessionState,
    t,
  } from '$lib';
  import { tagSuggestionShortcuts } from '$lib/config/shortcuts';

  let newTag = $state('');
  let allTags = $state<string[]>([]);
  let selectedIndex = $state(-1);
  let inputElement: HTMLInputElement | null = $state(null);

  /** Filters tags based on the provided search query. */
  const filterTags = (tags: string[], query: string) => {
    const search = query.trim().toLowerCase();
    if (!search) return tags;
    return tags.filter((t) => t.toLowerCase().includes(search));
  };

  /** Sorts tags, prioritizing active ones and then applying alphabetical order. */
  const sortTags = (tags: string[], active: string[]) => {
    return [...tags].sort((a, b) => {
      const aAdded = active.includes(a);
      const bAdded = active.includes(b);
      // Priority 1: Show tags already on the note at the top
      if (aAdded && !bAdded) return -1;
      if (!aAdded && bAdded) return 1;
      // Priority 2: Alphabetical order
      return a.localeCompare(b);
    });
  };

  let noteContent = $derived(sessionState.todayNoteContent);
  let currentTags = $derived(noteContent?.metadata.tags || []);
  let filteredTags = $derived(filterTags(allTags, newTag));
  let sortedTags = $derived(sortTags(filteredTags, currentTags));
  let suggestedTags = $derived(sortedTags.slice(0, 20));
  let activeTags = $derived(
    suggestedTags.filter((t) => currentTags.includes(t)),
  );
  let inactiveTags = $derived(
    suggestedTags.filter((t) => !currentTags.includes(t)),
  );

  /** Toggles a tag on or off for the current note. */
  const handleToggleTag = async (tagToToggle?: string) => {
    const tag = (tagToToggle || newTag).trim();
    if (!tag) return;

    const isRemoving = currentTags.includes(tag);
    const updatedContent = isRemoving
      ? await removeNoteTag(tag)
      : await addNoteTag(tag);

    if (updatedContent) {
      sessionState.todayNoteContent = updatedContent;
      // Refresh all tags to update frequency-based sorting from backend
      allTags = await getAllTags();
    }

    newTag = '';
    selectedIndex = -1;
  };

  /** Moves the keyboard selection index up or down. */
  const moveSelection = (direction: 'up' | 'down') => {
    const count = suggestedTags.length;
    if (count === 0) return;
    if (direction === 'down') {
      selectedIndex = (selectedIndex + 1) % count;
    } else {
      selectedIndex = (selectedIndex - 1 + count) % count;
    }
  };

  /** Selects the currently highlighted tag suggestion. */
  const selectCurrentSuggestion = () => {
    const tag = selectedIndex >= 0 ? suggestedTags[selectedIndex] : undefined;
    handleToggleTag(tag);
  };

  /** Removes the last tag from the note if the input field is empty. */
  const removeLastActiveTag = () => {
    if (!newTag && currentTags.length > 0) {
      handleToggleTag(currentTags[currentTags.length - 1]);
    }
  };

  /** Handles keyboard events for navigation and actions. */
  const handleKeyDown = (e: KeyboardEvent) => {
    // Check for shortcut combinations (Alt/Option + Physical Key)
    if (e.altKey && !e.metaKey && !e.ctrlKey) {
      const shortcutIndex = tagSuggestionShortcuts.codes.indexOf(e.code);

      if (shortcutIndex !== -1 && shortcutIndex < suggestedTags.length) {
        e.preventDefault();
        handleToggleTag(suggestedTags[shortcutIndex]);
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

  onMount(async () => {
    allTags = await getAllTags();
    inputElement?.focus();
  });
</script>

{#snippet tagItem(tag: string)}
  {@const isAdded = currentTags.includes(tag)}
  {@const globalIndex = suggestedTags.indexOf(tag)}
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
        <span class="mod">{inputManager.superLabel}</span>
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
      <svg
        class="search-icon"
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 -960 960 960"
        fill="currentColor"
      >
        <path
          d="M784-120 532-372q-30 24-69 38t-83 14q-109 0-184.5-75.5T120-580q0-109 75.5-184.5T380-840q109 0 184.5 75.5T640-580q0 44-14 83t-38 69l252 252-56 56ZM380-400q75 0 127.5-52.5T560-580q0-75-52.5-127.5T380-760q-75 0-127.5 52.5T200-580q0 75 52.5 127.5T380-400Z"
        />
      </svg>
      <input
        bind:this={inputElement}
        type="text"
        bind:value={newTag}
        onkeydown={handleKeyDown}
        oninput={() => (selectedIndex = -1)}
        placeholder={$t('tag.placeholder')}
        spellcheck="false"
      />
    </div>

    {#if suggestedTags.length > 0}
      <div class="suggestions">
        {#if activeTags.length > 0}
          <div class="section-label">{$t('tag.tags')}</div>
          {#each activeTags as tag}
            {@render tagItem(tag)}
          {/each}
        {/if}

        {#if inactiveTags.length > 0}
          <div class="section-label">{$t('tag.suggestions')}</div>
          {#each inactiveTags as tag}
            {@render tagItem(tag)}
          {/each}
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .tag-manager {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    min-height: 20rem;
  }

  .search-section {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .input-container {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 1rem;
    width: 1.25rem;
    height: 1.25rem;
    color: var(--text-muted);
    pointer-events: none;
  }

  input {
    width: 100%;
    background-color: var(--bg-surface);
    border: 0.0625rem solid var(--border);
    color: var(--text-main);
    padding: 0.875rem 1rem 0.875rem 3rem;
    border-radius: 0.75rem;
    font-size: 1rem;
    outline: none;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  }

  input:focus {
    border-color: var(--accent);
    background-color: var(--bg-base);
    box-shadow: 0 0 0 3px rgba(var(--accent-rgb, 0, 122, 255), 0.1);
  }

  .suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 0.5rem;
    background-color: var(--bg-base);
    border: 0.0625rem solid var(--border);
    border-radius: 0.75rem;
    box-shadow:
      0 10px 25px -5px rgba(0, 0, 0, 0.1),
      0 8px 10px -6px rgba(0, 0, 0, 0.1);
    z-index: 50;
    overflow: hidden;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .section-label {
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05rem;
    padding: 0.5rem 0.75rem;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.625rem 0.75rem;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-main);
    cursor: pointer;
    border-radius: 0.5rem;
    font-size: 0.9375rem;
    transition: all 0.15s ease;
  }

  .tag-label {
    flex: 1;
  }

  .shortcut-hint {
    font-family: var(--font-mono, monospace);
    font-size: 0.75rem;
    color: var(--text-muted);
    background-color: var(--bg-surface);
    border: 0.0625rem solid var(--border);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    display: flex;
    align-items: center;
    gap: 0.125rem;
    opacity: 0.6;
    pointer-events: none;
  }

  .suggestion-item:hover .shortcut-hint,
  .suggestion-item.selected .shortcut-hint {
    background-color: rgba(255, 255, 255, 0.15);
    border-color: transparent;
    color: var(--accent-text);
    opacity: 1;
  }

  .mod {
    font-size: 0.7rem;
    line-height: 1;
    opacity: 0.8;
    margin-right: 0.125rem;
  }

  .key {
    font-weight: 700;
    text-transform: uppercase;
  }

  .status-badge {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    color: var(--remove);
    opacity: 0;
    transition: opacity 0.2s;
  }

  .suggestion-item.is-added .status-badge {
    opacity: 1;
  }

  .hashtag {
    color: var(--text-muted);
    font-weight: 500;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background-color: var(--accent);
    color: var(--accent-text);
  }

  .suggestion-item:hover .hashtag,
  .suggestion-item.selected .hashtag,
  .suggestion-item:hover .status-badge,
  .suggestion-item.selected .status-badge {
    color: var(--accent-text);
    opacity: 0.8;
  }

  .suggestion-item.is-added:hover,
  .suggestion-item.is-added.selected {
    background-color: var(--remove);
    color: var(--accent-text);
  }
</style>
