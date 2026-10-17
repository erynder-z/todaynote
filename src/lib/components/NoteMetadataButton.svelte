<script lang="ts">
  /**
   * Dropdown item for showing the note metadata.
   * Triggers the onShowMetadata callback when clicked.
   */
  import { t } from '#lib';
  import type { NoteMetadataButtonProps } from '#lib/interfaces/notes.ts';

  let { onShowMetadata, closeDropdown = () => {} }: NoteMetadataButtonProps =
    $props();

  /**
   * Handles the click on the metadata button
   */
  const handleShowMetadata = (e: Event) => {
    e.stopPropagation();
    closeDropdown();
    onShowMetadata();
  };
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<button
  class="dropdown-item"
  role="menuitem"
  onclick={handleShowMetadata}
  title={$t('note.metadata.title')}
>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    height="1rem"
    viewBox="0 -960 960 960"
    width="1rem"
    fill="currentColor"
    ><path
      d="M440-280h80v-240h-80v240Zm40-320q17 0 28.5-11.5T520-640q0-17-11.5-28.5T480-680q-17 0-28.5 11.5T440-640q0 17 11.5 28.5T480-600Zm0 520q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Z"
    /></svg
  >
  <span>{$t('note.metadata.title')}</span>
</button>

<style>
  .dropdown-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    background: none;
    border: none;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    color: var(--text-ui);
    text-align: left;
    transition: background-color 0.15s;
  }

  .dropdown-item:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 90%);
    color: var(--accent);
  }

  .dropdown-item svg {
    flex-shrink: 0;
  }
</style>
