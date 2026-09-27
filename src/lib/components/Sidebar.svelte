<script lang="ts">
  /**
   * Control Center sidebar containing date, tags, and thread shortcuts.
   */

  import { ask } from '@tauri-apps/plugin-dialog';
  import { slide } from 'svelte/transition';
  import type { NoteContentResponse, NoteThread } from '$lib/interfaces/notes';
  import { toast } from '$lib/stores/toast.svelte';
  import { t } from '$lib/utils/i18n';
  import { notesService } from '$lib/utils/notes';
  import { useShortcuts } from '$lib/utils/shortcuts';
  import { sessionState } from '../stores/sessionState.svelte';
  import NoteDate from './NoteDate.svelte';
  import NoteDeleteButton from './NoteDeleteButton.svelte';
  import NoteTags from './NoteTags.svelte';
  import NoteThreadShortcuts from './NoteThreadShortcuts.svelte';
  import ThreadShortcutsModeToggle from './ThreadShortcutsModeToggle.svelte';

  let {
    noteContent,
    threads,
    notePath,
    onSelect,
    onNoteDeleted,
    width = 22, // rem
    isResizing = false,
  } = $props<{
    noteContent: NoteContentResponse | null;
    threads: NoteThread[];
    notePath: string | null;
    onSelect: (threadId: string) => void;
    onNoteDeleted: () => void;
    width?: number;
    isResizing?: boolean;
  }>();

  let noteOptionsOpen = $state(false);

  /**
   * Toggle sidebar visibility
   */
  const toggleSidebar = () => {
    sessionState.sidebarOpen = !sessionState.sidebarOpen;
  };

  /**
   * Toggle note options visibility
   */
  const toggleNoteOptions = (e: Event) => {
    e.stopPropagation();
    noteOptionsOpen = !noteOptionsOpen;
  };

  /**
   * Close note options
   */
  const closeNoteOptions = (e?: Event) => {
    if (e) e.stopPropagation();
    noteOptionsOpen = false;
  };

  /**
   * Sets the with the provided id as the selected thread
   */
  const handleThreadSelect = (threadId: string) => {
    if (sessionState.threadShortcutsMode === 'navigation') {
      onSelect(threadId);
    } else {
      const selectedThread = threads.find(
        (t: { id: string }) => t.id === threadId,
      );
      if (selectedThread) {
        sessionState.selectedThreadForOptions = selectedThread;
        sessionState.activePopup = 'threadOptions';
      }
    }
  };

  /**
   * Handles the deletion of the current note with confirmation.
   * Used by both the delete button and keyboard shortcut.
   */
  const handleDeleteNote = async () => {
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

  useShortcuts({
    toggleSidebar: () => toggleSidebar(),
    deleteNote: async () => await handleDeleteNote(),
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="sidebar"
  class:closed={!sessionState.sidebarOpen}
  class:resizing={isResizing}
  transition:slide={{ duration: 200, axis: 'x' }}
  style="width: {width}rem; --content-width: {width - 3}rem;"
  onclick={() => (noteOptionsOpen = false)}
>
  <button
    class="toggle-btn horizontal-only"
    onclick={toggleSidebar}
    title={$t('navigation.toggle_sidebar')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1rem"
      viewBox="0 -960 960 960"
      width="1rem"
      fill="currentColor"
      ><path d="M360-120v-720h80v720h-80Zm160-160v-400l200 200-200 200Z" /></svg
    >
  </button>

  <div class="sidebar-content">
    <div class="sidebar-section">
      <div class="note-header" onclick={(e) => e.stopPropagation()}>
        <NoteDate {noteContent} />
        <button
          class="note-options-btn"
          title={$t('note.options')}
          onclick={toggleNoteOptions}
          onkeydown={(e) => {
            if (e.key === 'Enter') toggleNoteOptions(e);
          }}
          aria-haspopup="true"
          aria-expanded={noteOptionsOpen}
          ><svg
            xmlns="http://www.w3.org/2000/svg"
            height="1.2rem"
            viewBox="0 -960 960 960"
            width="1.2rem"
            fill="currentColor"
            ><path
              d="M480-160q-33 0-56.5-23.5T400-240q0-33 23.5-56.5T480-320q33 0 56.5 23.5T560-240q0 33-23.5 56.5T480-160Zm0-240q-33 0-56.5-23.5T400-440q0-33 23.5-56.5T480-520q33 0 56.5 23.5T560-440q0 33-23.5 56.5T480-400Zm0-240q-33 0-56.5-23.5T400-640q0-33 23.5-56.5T480-720q33 0 56.5 23.5T560-640q0 33-23.5 56.5T480-560Z"
            /></svg
          >
        </button>
        {#if noteOptionsOpen}
          <!-- svelte-ignore a11y_interactive_supports_focus -->
          <div
            class="note-options-dropdown"
            role="menu"
            onclick={(e) => e.stopPropagation()}
          >
            <NoteDeleteButton
              {noteContent}
              {notePath}
              onDelete={handleDeleteNote}
              closeDropdown={closeNoteOptions}
            />
          </div>
        {/if}
      </div>
    </div>
    <div class="sidebar-section">
      <h3 class="sidebar-title">{$t('search.tags')}</h3>
      <NoteTags {noteContent} />
    </div>

    <div class="sidebar-section">
      <div class="threads-title-container">
        <h3 class="sidebar-title">{$t('search.threads')}</h3>
        <ThreadShortcutsModeToggle />
      </div>
      <NoteThreadShortcuts {threads} onSelect={handleThreadSelect} />
    </div>
  </div>
</div>

<style>
  .sidebar {
    height: 100%;
    flex-shrink: 0;
    padding: 3rem 1.5rem;
    background-color: color-mix(in srgb, var(--bg-surface), black 10%);
    box-shadow: 0 1px 25px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    position: relative;
  }

  .sidebar-content {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: var(--content-width);
    flex-shrink: 0;
  }

  .toggle-btn {
    position: absolute;
    top: 50%;
    left: 0;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-ui-muted);
    padding: 0.75rem 0.25rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0 0.5rem 0.5rem 0;
    transition:
      padding 0.2s,
      background-color 0.2s,
      color 0.2s;
    z-index: 10;
  }

  .toggle-btn:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
    border-color: var(--accent);
  }

  .sidebar-section {
    margin-top: 2rem;
  }

  .sidebar-title {
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-ui-muted);
    margin-bottom: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.15em;
  }

  .threads-title-container {
    display: flex;
    justify-content: space-between;
  }

  .note-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    position: relative;
  }

  .note-options-btn {
    background: none;
    border: none;
    color: var(--text-ui-muted);
    padding: 0.25rem;
    cursor: pointer;
    border-radius: 0.25rem;
    transition:
      background-color 0.2s,
      color 0.2s;
  }

  .note-options-btn:hover {
    background-color: color-mix(in srgb, var(--accent), transparent 85%);
    color: var(--accent);
  }

  .note-options-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .note-options-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 0.25rem;
    background-color: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 100;
    overflow: hidden;
  }

  @media (min-width: 1025px) {
    .sidebar {
      opacity: 1;
    }

    .sidebar.resizing {
      transition: none !important;
    }
  }

  @media (min-width: 1025px) {
    .sidebar {
      transition:
        width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        padding 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        box-shadow 0.3s cubic-bezier(0.4, 0, 0.2, 1);
      opacity: 1;
    }

    .sidebar.resizing {
      transition: none !important;
    }

    .sidebar.closed {
      width: 0 !important;
      padding-left: 0 !important;
      padding-right: 0 !important;
      opacity: 0;
      box-shadow: none;
      overflow: hidden;
      pointer-events: none;
    }
  }
</style>
