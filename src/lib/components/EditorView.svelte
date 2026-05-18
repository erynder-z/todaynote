<script lang="ts">
  /**
   * View component that combines the note editor with the note sidebar.
   */
  import type { NoteContentResponse } from '$lib/interfaces/notes';
  import { settings } from '../index';
  import { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import NoteControlCenter from './NoteControlCenter.svelte';
  import NoteEditor from './NoteEditor.svelte';

  let { noteContent = $bindable(), notePath } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
  }>();

  const editor = new EditorStore();

  /**
   * Indicates whether the sidebar is currently being resized.
   */
  let isResizing = $state(false);

  $effect(() => {
    editor.sync(noteContent, notePath);
  });

  /**
   * Handles navigation to a specific thread, creating it if it doesn't exist.
   */
  const handleJump = async (name: string) => {
    // Update content to ensure thread exists
    if (!editor.threads.some((s) => s.name === name)) {
      const updatedContent = await editor.ensureThreadExists(name);
      noteContent = updatedContent;
    }

    if (editor.jumpToThread) editor.jumpToThread(name);

    // Close sidebar in vertical layout after jumping
    sessionState.sidebarOpen = false;
  };

  /**
   * Initiates the resizing process for the sidebar.
   */
  const startResizing = () => {
    isResizing = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  };

  /**
   * Concludes the resizing process and persists the new width.
   */
  const stopResizing = () => {
    if (!isResizing) return;
    isResizing = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    settings.setControlCenterWidth(settings.controlCenterWidth);
  };

  /**
   * Updates the sidebar width based on mouse movement during resizing.
   */
  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizing) return;

    const minWidth = 250;
    const maxWidth = 600;
    const newWidth = window.innerWidth - e.clientX;

    if (newWidth >= minWidth && newWidth <= maxWidth) {
      settings.controlCenterWidth = newWidth;
    }
  };
</script>

<svelte:window onmousemove={handleMouseMove} onmouseup={stopResizing} />

<div class="note-container">
  <div class="note-layout">
    <div class="editor-main">
      <NoteEditor bind:noteContent {notePath} {editor} />
    </div>

    <!-- Resizer Handle -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resizer"
      class:resizing={isResizing}
      onmousedown={startResizing}
    ></div>

    {#if sessionState.sidebarOpen}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="vertical-layout"
        onclick={() => (sessionState.sidebarOpen = false)}
      ></div>
    {/if}

    <div class="sidebar-wrapper" class:open={sessionState.sidebarOpen}>
      <NoteControlCenter
        {noteContent}
        threads={editor.threads}
        onSelect={handleJump}
        width={settings.controlCenterWidth}
      />
    </div>
  </div>
</div>

<style>
  .note-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-base);
  }

  .note-layout {
    display: flex;
    flex: 1;
    gap: 0;
    min-height: 0;
    position: relative;
  }

  .editor-main {
    flex: 1;
    min-width: 0;
    padding: 3rem;
    overflow-y: auto;
    background-color: var(--bg-base);
  }

  .sidebar-wrapper {
    display: contents;
  }

  .resizer {
    width: 4px;
    height: 100%;
    cursor: col-resize;
    background-color: transparent;
    transition: background-color 0.2s;
    z-index: 10;
    margin-left: -2px;
    margin-right: -2px;
  }

  .resizer:hover,
  .resizer.resizing {
    background-color: var(--accent);
  }

  .vertical-layout {
    display: none;
  }

  @media (max-width: 1024px) {
    .editor-main {
      padding: 1.5rem;
    }

    .resizer {
      display: none;
    }

    .sidebar-wrapper {
      display: block;
      position: absolute;
      top: 0;
      right: 0;
      height: 100%;
      z-index: 1000;
      transform: translateX(100%);
      transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
      will-change: transform;
    }

    .sidebar-wrapper.open {
      transform: translateX(0);
    }

    .vertical-layout {
      display: block;
      position: absolute;
      inset: 0;
      background: rgba(0, 0, 0, 0.4);
      backdrop-filter: blur(2px);
      z-index: 999;
    }
  }
</style>
