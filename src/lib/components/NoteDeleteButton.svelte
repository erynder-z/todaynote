<script lang="ts">
  /**
   * Button for deleting a note.
   * Shows a native confirmation dialog before deletion.
   * Handles all deletion logic including closing dropdown and navigation.
   */
  import { ask } from '@tauri-apps/plugin-dialog';
  import { get } from 'svelte/store';
  import { locale, t, toast } from '$lib';
  import type { NoteContentResponse } from '$lib/interfaces/notes';
  import { notesService } from '$lib/utils/notes';

  let {
    noteContent,
    notePath,
    onNoteDeleted,
    closeDropdown = () => {},
  } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
    onNoteDeleted: () => void;
    closeDropdown?: () => void;
  }>();

  /**
   * Handles the deletion of the current note via Tauri dialog
   */
  const handleDelete = async (e: Event) => {
    e.stopPropagation();

    if (!notePath) return;

    const confirmed = await ask($t('note.delete_confirm_message'), {
      title: $t('note.delete_confirm_title'),
      kind: 'warning',
      okLabel: $t('note.delete_confirm'),
      cancelLabel: $t('note.delete_cancel'),
    });

    if (!confirmed) return;

    try {
      const success = await notesService.deleteNote(notePath);

      if (success) {
        closeDropdown();
        onNoteDeleted();
        toast.success($t('note.delete_success'));
      } else {
        toast.error($t('note.delete_error'));
      }
    } catch (error) {
      console.error('Failed to delete note:', error);
      toast.error($t('note.delete_error'));
    }
  };
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<button
  class="dropdown-item"
  role="menuitem"
  onclick={handleDelete}
  title={$t('note.delete')}
>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    height="1rem"
    viewBox="0 -960 960 960"
    width="1rem"
    fill="currentColor"
    ><path
      d="m376-300 104-104 104 104 56-56-104-104 104-104-56-56-104 104-104-104-56 56 104 104-104 104 56 56Zm-96 180q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520Zm-400 0v520-520Z"
    /></svg
  >
  <span>{$t('note.delete')}</span>
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
    background-color: color-mix(in srgb, var(--error), transparent 90%);
    color: var(--error);
  }

  .dropdown-item svg {
    flex-shrink: 0;
  }
</style>
