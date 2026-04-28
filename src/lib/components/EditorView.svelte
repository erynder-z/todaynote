<script lang="ts">
  /**
   * View component that combines the note editor with the note sidebar.
   */
  import type { NoteContentResponse } from '$lib/types/notes';
  import { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import NoteControlCenter from './NoteControlCenter.svelte';
  import NoteEditor from './NoteEditor.svelte';

  let { noteContent = $bindable(), notePath } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
  }>();

  const editor = new EditorStore();

  $effect(() => {
    editor.sync(noteContent, notePath);
  });

  /**
   * Handles navigation to a specific section, creating it if it doesn't exist.
   */
  const handleJump = async (name: string) => {
    // Update content to ensure section exists
    if (!editor.sections.some((s) => s.name === name)) {
      const updatedContent = await editor.ensureSectionExists(name);
      noteContent = updatedContent;
    }

    if (editor.jumpToSection) editor.jumpToSection(name);

    // Close sidebar in vertical layout after jumping
    sessionState.sidebarOpen = false;
  };
</script>

<div class="note-container">
  <div class="note-layout">
    <div class="editor-main">
      <NoteEditor bind:noteContent {notePath} {editor} />
    </div>

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
        sections={editor.sections}
        onSelect={handleJump}
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

  .vertical-layout {
    display: none;
  }

  @media (max-width: 1024px) {
    .editor-main {
      padding: 1.5rem;
    }

    .sidebar-wrapper {
      display: block;
      position: absolute;
      top: 0;
      right: 0;
      height: 100%;
      z-index: 1000;
      transform: translateX(100%);
      transition: transform 0.3s ease;
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
