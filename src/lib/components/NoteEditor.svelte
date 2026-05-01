<script lang="ts">
  /**
   * The main note editor component. Displays a note's content in an editable form.
   */
  import {
    defaultValueCtx,
    Editor,
    editorViewCtx,
    parserCtx,
    rootCtx,
  } from '@milkdown/core';
  import { listener, listenerCtx } from '@milkdown/plugin-listener';
  import { commonmark } from '@milkdown/preset-commonmark';
  import { gfm } from '@milkdown/preset-gfm';
  import { keymap } from '@milkdown/prose/keymap';
  import { Selection } from '@milkdown/prose/state';
  import { $prose as prosePlugin } from '@milkdown/utils';
  import { untrack } from 'svelte';
  import type { NoteContentResponse, NoteSection } from '$lib/types/notes';
  import { tagSuggestionShortcuts } from '../config/shortcuts';
  import { EditorStore } from '../stores/editor.svelte';
  import { sessionState } from '../stores/sessionState.svelte';
  import { jumpToSectionInEditor } from '../utils/editor';
  import { useShortcuts } from '../utils/shortcuts';

  // --- Props & State ---

  let {
    noteContent = $bindable(),
    notePath,
    editor,
  } = $props<{
    noteContent: NoteContentResponse | null;
    notePath: string | null;
    editor: EditorStore;
  }>();

  let editorContainer: HTMLDivElement | null = $state(null);
  let milkdownInstance: Editor | null = $state(null);

  // --- Lifecycle & Reactive Effects ---

  /**
   * 1. Sync props to the internal store before rendering
   */
  $effect.pre(() => {
    editor.sync(noteContent, notePath);
  });

  /**
   * 2. Initialize the Milkdown editor instance
   */
  $effect(() => {
    if (!editorContainer || !notePath) return;

    let isDestroyed = false;
    createEditor(editorContainer).then((instance) => {
      if (isDestroyed) {
        instance.destroy();
      } else {
        milkdownInstance = instance;
      }
    });

    return () => {
      isDestroyed = true;
      milkdownInstance?.destroy();
      milkdownInstance = null;
    };
  });

  /**
   * 3. Coordinate reactive updates (content sync and focus)
   */
  $effect(() => {
    const instance = milkdownInstance;
    if (!instance) return;

    if (editor.pendingExternalUpdate) {
      updateEditorContent(instance, editor.content);
      editor.pendingExternalUpdate = false;
    }

    if (sessionState.activePopup === null && notePath)
      untrack(() => focusEditor(instance));
  });

  // --- Actions & Helpers ---

  /**
   * Creates the Milkdown editor instance with required plugins and config.
   */
  const createEditor = (container: HTMLDivElement) => {
    return Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, container);
        ctx.set(
          defaultValueCtx,
          untrack(() => editor.content),
        );
        ctx.get(listenerCtx).markdownUpdated((_, markdown) => {
          editor.updateContent(markdown);
        });
      })
      .use(commonmark)
      .use(gfm)
      .use(listener)
      .use(customKeymap)
      .create();
  };

  /**
   * Updates the editor's content from a Markdown string and positions the cursor.
   */
  const updateEditorContent = (instance: Editor, markdown: string) => {
    instance.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const parser = ctx.get(parserCtx);
      const doc = parser(markdown);
      if (!doc) return;

      let tr = view.state.tr.replaceWith(0, view.state.doc.content.size, doc);

      // Ensure trailing empty line for headings (Milkdown parser workaround)
      if (doc.lastChild?.type.name === 'heading') {
        const paragraph = view.state.schema.nodes.paragraph.create();
        tr = tr.insert(tr.doc.content.size, paragraph);
      }

      // Position cursor at end and focus
      const selection = Selection.atEnd(tr.doc);
      view.dispatch(tr.setSelection(selection).scrollIntoView());
      view.focus();
    });
  };

  /**
   * Focuses the editor instance.
   */
  const focusEditor = (instance: Editor) => {
    instance.action((ctx) => ctx.get(editorViewCtx).focus());
  };

  /**
   * Main entry point for jumping to a section.
   */
  const handleJump = async (name: string) => {
    const instance = milkdownInstance;
    if (!instance) return;

    const exists = editor.sections.some((s: NoteSection) => s.name === name);
    if (exists) {
      jumpToSectionInEditor(instance, name);
    } else {
      await editor.ensureSectionExists(name);
      setTimeout(() => jumpToSectionInEditor(instance, name), 100);
    }
  };

  /**
   * Jumps to a section based on its index (0-8).
   */
  const jumpToSectionByIndex = async (idx: number) => {
    const section = editor.sections[idx];
    if (section?.name) await handleJump(section.name);
  };

  useShortcuts({
    focusLastLine: () => {
      if (sessionState.activePopup !== null) return false;
      if (milkdownInstance) focusEditor(milkdownInstance);
    },
    jumpByNumber: (e) => {
      if (sessionState.activePopup !== null) return false;

      const idx = tagSuggestionShortcuts.codes.indexOf(e.code);
      if (idx !== -1 && idx < editor.sections.length) {
        jumpToSectionByIndex(idx);
      }
    },
  });

  const customKeymap = prosePlugin(() =>
    keymap({
      'Mod-1': () => true,
      'Mod-2': () => true,
    }),
  );

  // Connect the store's sync back to the component's bindable props
  $effect(() => {
    editor.onJump = (updated: NoteContentResponse) => (noteContent = updated);
  });

  // Expose jump functionality to parent components
  $effect(() => {
    editor.jumpToSection = (name: string) => {
      const instance = milkdownInstance;
      if (instance) jumpToSectionInEditor(instance, name);
    };
  });
</script>

<div bind:this={editorContainer} class="milkdown-editor-wrapper"></div>

<style>
  .milkdown-editor-wrapper :global(.milkdown) {
    width: 100%;
    min-height: 70dvh;
    background: transparent;
    color: var(--text-main);
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.6;
  }

  .milkdown-editor-wrapper :global(.milkdown .editor) {
    outline: none;
    padding-bottom: 5rem;
  }

  /* Headings */
  .milkdown-editor-wrapper :global(.milkdown h1),
  .milkdown-editor-wrapper :global(.milkdown h2),
  .milkdown-editor-wrapper :global(.milkdown h3),
  .milkdown-editor-wrapper :global(.milkdown h4) {
    font-weight: 600;
    margin-top: 2rem;
    margin-bottom: 1rem;
    line-height: 1.3;
  }

  .milkdown-editor-wrapper :global(.milkdown h1) {
    color: var(--md-h1);
    font-size: 1.875rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.5rem;
  }
  .milkdown-editor-wrapper :global(.milkdown h2) {
    color: var(--md-h2);
    font-size: 1.5rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.3rem;
  }
  .milkdown-editor-wrapper :global(.milkdown h3) {
    color: var(--md-h3);
    font-size: 1.25rem;
  }
  .milkdown-editor-wrapper :global(.milkdown h4) {
    color: var(--md-h4);
    font-size: 1.125rem;
  }

  /* Links */
  .milkdown-editor-wrapper :global(.milkdown a) {
    color: var(--md-link);
    text-decoration: underline;
    text-underline-offset: 2px;
    transition: color 0.2s ease;
  }

  .milkdown-editor-wrapper :global(.milkdown a:hover) {
    color: var(--md-link-hover);
  }

  /* Lists */
  .milkdown-editor-wrapper :global(.milkdown ul),
  .milkdown-editor-wrapper :global(.milkdown ol) {
    padding-left: 2rem;
    margin: 1rem 0;
  }

  .milkdown-editor-wrapper :global(.milkdown li) {
    margin: 0.5rem 0;
  }

  .milkdown-editor-wrapper :global(.milkdown li::marker) {
    color: var(--md-list-marker);
    font-weight: 600;
  }

  .milkdown-editor-wrapper :global(.milkdown ul) {
    list-style-type: disc;
  }

  .milkdown-editor-wrapper :global(.milkdown ol) {
    list-style-type: decimal;
  }

  /* Code */
  .milkdown-editor-wrapper :global(.milkdown pre) {
    background: var(--md-code-bg);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    padding: 1rem;
    overflow-x: auto;
    margin: 1.5rem 0;
    font-family: var(--font-mono);
    font-size: 0.875rem;
    line-height: 1.5;
  }

  .milkdown-editor-wrapper :global(.milkdown pre code) {
    background: transparent;
    color: var(--md-code-text);
    padding: 0;
    border-radius: 0;
    font-size: inherit;
  }

  .milkdown-editor-wrapper :global(.milkdown code) {
    background: var(--md-code-inline-bg);
    color: var(--md-code-text);
    padding: 0.2rem 0.4rem;
    border-radius: 0.25rem;
    font-family: var(--font-mono);
    font-size: 0.875rem;
  }

  /* Blockquotes */
  .milkdown-editor-wrapper :global(.milkdown blockquote) {
    border-left: 4px solid var(--md-quote-border);
    padding-left: 1.5rem;
    margin: 1.5rem 0;
    color: var(--text-muted);
    font-style: italic;
  }

  /* Horizontal Rule */
  .milkdown-editor-wrapper :global(.milkdown hr) {
    border: 0;
    border-top: 1px solid var(--md-hr);
    margin: 2.5rem 0;
  }

  /* Spacing */
  .milkdown-editor-wrapper :global(.milkdown p) {
    margin: 1rem 0;
  }

  /* Strong/Emphasis */
  .milkdown-editor-wrapper :global(.milkdown strong) {
    font-weight: 600;
    color: var(--md-bold);
  }

  .milkdown-editor-wrapper :global(.milkdown em) {
    font-style: italic;
    color: var(--md-italic);
  }

  /* Text Selection */
  .milkdown-editor-wrapper :global(.milkdown ::selection) {
    background: var(--text-selection);
    color: var(--text-main);
  }
</style>
