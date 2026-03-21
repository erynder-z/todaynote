<script lang="ts">
  /**
   * Component for displaying and managing tags for a note.
   */

  import { onMount } from 'svelte';
  import { inputManager, sessionState, t, useShortcuts } from '$lib';
  import type { NoteContentResponse } from '$lib/types/notes';
  import { addNoteTag, getAllTags, removeNoteTag } from '$lib/utils/notes';

  let { noteContent } = $props<{
    noteContent: NoteContentResponse | null;
  }>();

  let tags = $derived(noteContent?.metadata.tags || []);
  let isAddingTag = $state(false);
  let tagToRemove = $state<string | null>(null);
  let newTag = $state('');
  let allTags = $state<string[]>([]);
  let selectedIndex = $state(-1);

  let suggestedTags = $derived(
    allTags
      .filter(
        (tag) =>
          !tags.includes(tag) &&
          tag.toLowerCase().includes(newTag.trim().toLowerCase()),
      )
      .slice(0, 8),
  );

  // Reset selection when input changes
  $effect(() => {
    newTag;
    selectedIndex = -1;
  });

  // Close input when clicking outside
  onMount(() => {
    const handleClick = () => (tagToRemove = null);
    window.addEventListener('click', handleClick);
    return () => window.removeEventListener('click', handleClick);
  });

  /**
   * Closes the tag input field.
   */
  const closeInput = () => {
    isAddingTag = false;
    newTag = '';
    selectedIndex = -1;
  };

  /**
   * Triggers the addition of a new tag and updates the session state.
   */
  const handleAddTag = async (tagToAdd?: string) => {
    const finalTag = tagToAdd || newTag.trim();
    if (finalTag && !tags.includes(finalTag)) {
      const updatedContent = await addNoteTag(finalTag);
      if (updatedContent) sessionState.todayNoteContent = updatedContent;
    }
    closeInput();
  };

  /**
   * Triggers the removal of a tag and updates the session state.
   */
  const handleRemoveTag = async (e: MouseEvent) => {
    e.stopPropagation();
    if (tagToRemove) {
      const updatedContent = await removeNoteTag(tagToRemove);
      if (updatedContent) sessionState.todayNoteContent = updatedContent;
    }
    tagToRemove = null;
  };

  /**
   * Handles click on a tag pill. Sets the tag for removal if Shift is held.
   */
  const handleTagClick = (e: MouseEvent, tag: string) => {
    if (inputManager.shiftPressed) {
      e.preventDefault();
      e.stopPropagation();
      tagToRemove = tag;
      isAddingTag = false;
    }
  };

  /**
   * Loads all tags and shows initial suggestions.
   */
  const startAddingTag = async () => {
    tagToRemove = null;
    isAddingTag = true;
    allTags = await getAllTags();
  };

  useShortcuts({ addTag: () => startAddingTag() });

  /**
   * Moves the selection down in the suggestions list.
   */
  const moveSelectionDown = () => {
    selectedIndex =
      selectedIndex < suggestedTags.length - 1 ? selectedIndex + 1 : -1;
  };

  /**
   * Moves the selection up in the suggestions list.
   */
  const moveSelectionUp = () => {
    selectedIndex =
      selectedIndex > -1 ? selectedIndex - 1 : suggestedTags.length - 1;
  };

  /**
   * Handles keyboard interactions for the tag input field.
   */
  const handleKeyDown = (e: KeyboardEvent): void => {
    switch (e.key) {
      case 'Enter':
        handleAddTag(suggestedTags[selectedIndex]);
        break;
      case 'Escape':
        closeInput();
        break;
      case 'ArrowDown':
        e.preventDefault();
        moveSelectionDown();
        break;
      case 'ArrowUp':
        e.preventDefault();
        moveSelectionUp();
        break;
      default:
        break;
    }
  };
</script>

<div class="tags-container">
  {#each tags as tag}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <span
      class="tag-pill"
      class:to-remove={tagToRemove === tag}
      onclick={(e) => handleTagClick(e, tag)}
    >
      {tag}
    </span>
  {/each}

  <div class="add-tag-wrapper">
    {#if tagToRemove}
      <button
        class="add-tag-btn remove"
        onclick={handleRemoveTag}
        aria-label="Remove tag"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1rem"
          viewBox="0 -960 960 960"
          width="1rem"
          fill="currentColor"
          ><path
            d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
          /></svg
        >
      </button>
    {:else if isAddingTag}
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        class="tag-input"
        bind:value={newTag}
        onkeydown={handleKeyDown}
        onblur={() => setTimeout(closeInput, 200)}
        autofocus
        placeholder={$t('tag.placeholder')}
      />

      {#if suggestedTags.length > 0}
        <div class="suggestions">
          {#each suggestedTags as tag, i}
            <button
              class="suggestion-item"
              class:selected={i === selectedIndex}
              onclick={() => handleAddTag(tag)}
              onmouseenter={() => (selectedIndex = i)}
            >
              {tag}
            </button>
          {/each}
        </div>
      {/if}
    {:else}
      <button class="add-tag-btn" onclick={startAddingTag} aria-label="Add tag">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1rem"
          viewBox="0 -960 960 960"
          width="1rem"
          fill="currentColor"
          ><path
            d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z"
          /></svg
        >
      </button>
    {/if}
  </div>
</div>

<style>
  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
  }

  .tag-pill {
    font-size: 0.75rem;
    background-color: var(--accent);
    color: var(--accent-text);
    padding-inline: 0.625rem;
    padding-block: 0.125rem;
    border-radius: 999rem;
    font-weight: 600;
    white-space: nowrap;
    transition: all 0.2s ease;
    cursor: default;
  }

  .tag-pill.to-remove {
    background-color: var(--remove);
    opacity: 0.8;
  }

  .add-tag-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .add-tag-btn {
    font-size: 0.75rem;
    background-color: var(--bg-surface);
    color: var(--accent);
    border: 0.0625rem dashed var(--border);
    padding-inline: 0.5rem;
    padding-block: 0.125rem;
    border-radius: 999rem;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .add-tag-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .add-tag-btn.remove {
    color: var(--remove);
    border-color: var(--remove);
    border-style: solid;
  }

  .tag-input {
    font-size: 0.75rem;
    background-color: var(--bg-surface);
    color: var(--text-main);
    border: 0.0625rem solid var(--accent);
    padding-inline: 0.625rem;
    padding-block: 0.125rem;
    border-radius: 999rem;
    outline: none;
    width: 6rem;
  }

  .suggestions {
    position: absolute;
    top: calc(100% + 0.25rem);
    left: 0;
    z-index: 999;
    background-color: var(--bg-base);
    border: 0.0625rem solid var(--border);
    border-radius: 0.5rem;
    box-shadow: 0 0.25rem 0.75rem rgba(0, 0, 0, 0.1);
    min-width: 8rem;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .suggestion-item {
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-main);
    cursor: pointer;
    width: 100%;
    transition: background-color 0.1s ease;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background-color: var(--accent);
    color: var(--accent-text);
  }
</style>
