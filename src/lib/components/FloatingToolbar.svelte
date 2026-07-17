<script lang="ts">
  /**
   * Floating selection toolbar for native inline formatting (bold, italic, strikethrough, code, link).
   * Appears above text selection inside the Milkdown editor.
   */

  import type { Editor } from '@milkdown/core';
  import { editorViewCtx } from '@milkdown/core';
  import type { Ctx } from '@milkdown/ctx';
  import { toggleMark } from '@milkdown/prose/commands';
  import type { Mark, Node as PMNode } from 'prosemirror-model';
  import type { EditorState } from 'prosemirror-state';
  import { fade, fly } from 'svelte/transition';

  let { editorInstance } = $props<{ editorInstance: Editor | null }>();

  let visible = $state(false);
  let x = $state(0);
  let y = $state(0);
  let container: HTMLDivElement | null = $state(null);
  let isBold = $state(false);
  let isItalic = $state(false);
  let isStrikethrough = $state(false);
  let isCode = $state(false);
  let isLink = $state(false);
  let showLinkInput = $state(false);
  let linkUrl = $state('');
  let inputElement = $state<HTMLInputElement | null>(null);
  let isSelecting = $state(false);
  let isKeyboardSelecting = $state(false);
  let pendingSelection: Selection | null = $state(null);
  let lastSelectionRange: string | null = $state(null);
  let isMakingNewSelection = $state(false);
  let shouldPreventToolbarReopen = $state(false);

  /**
   * Updates state of marks in the current selection
   */
  const updateSelectionState = () => {
    if (!editorInstance) return;
    try {
      editorInstance.action((ctx: Ctx) => {
        const view = ctx.get(editorViewCtx);
        const { state } = view;
        const { from, to, $from: fromPos } = state.selection;

        // Toggle state indicators for basic markdown styles
        isBold =
          state.doc.rangeHasMark(from, to, state.schema.marks.strong) ||
          fromPos.marks().some((m: Mark) => m.type.name === 'strong');
        isItalic =
          state.doc.rangeHasMark(from, to, state.schema.marks.emphasis) ||
          fromPos.marks().some((m: Mark) => m.type.name === 'emphasis');
        isStrikethrough =
          state.doc.rangeHasMark(from, to, state.schema.marks.strike_through) ||
          fromPos.marks().some((m: Mark) => m.type.name === 'strike_through');
        isCode =
          state.doc.rangeHasMark(from, to, state.schema.marks.inlineCode) ||
          fromPos.marks().some((m: Mark) => m.type.name === 'inlineCode');
        isLink =
          state.doc.rangeHasMark(from, to, state.schema.marks.link) ||
          fromPos.marks().some((m: Mark) => m.type.name === 'link');

        // Extract active link URL if we're not currently editing it
        if (!showLinkInput) {
          const linkMark =
            fromPos.marks().find((m: Mark) => m.type.name === 'link') ||
            (from !== to &&
            state.doc.rangeHasMark(from, to, state.schema.marks.link)
              ? findLinkMarkInRange(state, from, to)
              : null);
          linkUrl = linkMark ? linkMark.attrs.href || '' : '';
        }
      });
    } catch (e) {
      console.error('Error updating selection state', e);
    }
  };

  /**
   * Finds the first link mark in a given range
   */
  const findLinkMarkInRange = (
    state: EditorState,
    from: number,
    to: number,
  ) => {
    let found: Mark | null = null;
    state.doc.nodesBetween(from, to, (node: PMNode) => {
      const m = node.marks.find((m: Mark) => m.type.name === 'link');
      if (m) found = m;
    });
    return found;
  };

  /**
   * Toggles standard Prosemirror marks
   */
  const toggleFormat = (markName: string) => {
    if (!editorInstance) return;

    editorInstance.action((ctx: Ctx) => {
      const view = ctx.get(editorViewCtx);
      const markType = view.state.schema.marks[markName];
      if (!markType) return;

      const { state, dispatch } = view;
      toggleMark(markType)(state, dispatch);
      toggleMark(markType)(state, dispatch);
      updateSelectionState();
    });
  };

  /**
   * Applies or removes a hyperlink
   */
  const applyLink = (url: string) => {
    if (!editorInstance) return;

    editorInstance.action((ctx: Ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state, dispatch } = view;
      const { from, to } = state.selection;
      if (from === to) return;

      const markType = state.schema.marks.link;
      if (!markType) return;

      let tr = state.tr;
      if (!url) {
        tr = tr.removeMark(from, to, markType);
      } else {
        tr = tr.removeMark(from, to, markType);
        tr = tr.addMark(from, to, markType.create({ href: url }));
      }
      dispatch(tr);

      showLinkInput = false;
      updateSelectionState();
    });
  };

  /**
   * Creates a unique identifier for a selection to detect changes
   */
  const getSelectionKey = (selection: Selection): string => {
    if (selection.rangeCount === 0) return '';

    const range = selection.getRangeAt(0);
    return `${range.startContainer.textContent?.substring(0, 20) || ''}:${range.startOffset}:${range.endOffset}`;
  };

  /**
   * Checks if the current selection is valid (non-empty and not collapsed).
   */
  const isSelectionValid = (selection: Selection | null): boolean => {
    if (!selection) return false;

    return !selection.isCollapsed && !!selection.toString().trim();
  };

  /**
   * Checks if the selection is inside the Milkdown editor.
   */
  const isSelectionInEditor = (selection: Selection | null): boolean => {
    if (!selection?.anchorNode) return false;

    let node: Node | null = selection.anchorNode;
    while (node) {
      if (
        node instanceof HTMLElement &&
        node.classList.contains('milkdown-editor-wrapper')
      ) {
        return true;
      }
      node = node.parentNode;
    }
    return false;
  };

  /**
   * Positions the toolbar above the current selection with smart viewport bounding.
   * Uses CSS transforms for smooth positioning while ensuring the toolbar stays visible.
   */
  const positionToolbar = (selection: Selection) => {
    if (selection.rangeCount === 0) {
      visible = false;
      return;
    }

    const range = selection.getRangeAt(0);
    const rect = range.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    // Calculate ideal position (centered above selection)
    let idealX = rect.left + rect.width / 2;
    let idealY = rect.top - 10;

    // Get toolbar dimensions
    const toolbarWidth = 250;
    const toolbarHeight = 40;

    // Calculate bounds with safe margins
    const safeMargin = 12; // Minimum distance from viewport edges
    const minX = toolbarWidth / 2 + safeMargin;
    const maxX = viewportWidth - toolbarWidth / 2 - safeMargin;

    // Constrain X position to stay within viewport
    const constrainedX = Math.max(minX, Math.min(idealX, maxX));

    // For very small screens, center the toolbar
    if (maxX < minX) {
      x = viewportWidth / 2;
    } else {
      x = constrainedX;
    }

    // Position above selection, but ensure it's visible
    // Calculate position above with natural spacing
    const spacing = 8; // Space between text and toolbar
    let positionY = idealY - toolbarHeight - spacing;

    // If not enough space above, position below
    if (positionY < safeMargin) {
      y = rect.bottom + spacing;
    } else {
      y = idealY; // CSS will handle the -100% transform
    }

    // Final vertical bounds check
    const minY = safeMargin;
    const maxY = viewportHeight - toolbarHeight - safeMargin;

    if (y < minY) {
      y = minY + toolbarHeight + spacing; // Force below if too close to top
    } else if (y > maxY) {
      y = maxY; // Constrain to bottom bound
    }
  };

  /**
   * Handles selection changes to prepare toolbar positioning but delays showing it.
   */
  const handleSelectionChange = () => {
    if (showLinkInput) return;

    const selection = window.getSelection();

    // Early return if no selection
    if (!selection) {
      visible = false;
      pendingSelection = null;
      lastSelectionRange = null;
      return;
    }

    // Check if selection is valid
    if (!isSelectionValid(selection)) {
      visible = false;
      pendingSelection = null;
      lastSelectionRange = null;
      return;
    }

    // Check if selection is in editor
    if (!isSelectionInEditor(selection)) {
      visible = false;
      pendingSelection = null;
      lastSelectionRange = null;
      return;
    }

    const currentSelectionKey = getSelectionKey(selection);

    // If this is the same selection as before (clicking inside existing selection),
    // set a flag to prevent toolbar from reopening
    if (currentSelectionKey === lastSelectionRange) {
      visible = false;
      isMakingNewSelection = false;
      shouldPreventToolbarReopen = true;
      return;
    }

    // Reset the prevention flag for new selections
    shouldPreventToolbarReopen = false;

    // Store the selection for later display but don't show yet
    pendingSelection = selection;
    lastSelectionRange = currentSelectionKey;
    positionToolbar(selection);
    updateSelectionState();

    // Hide toolbar while selecting
    visible = false;
  };

  /**
   * Shows the toolbar with the pending selection after mouse/key release
   */
  const showPendingToolbar = () => {
    if (pendingSelection && !isSelecting && !isKeyboardSelecting) {
      // Get fresh selection in case it changed
      const currentSelection = window.getSelection();
      if (
        currentSelection &&
        !currentSelection.isCollapsed &&
        isSelectionInEditor(currentSelection)
      ) {
        positionToolbar(currentSelection);
        setTimeout(() => {
          visible = true;
        }, 50);
      }
    }
  };

  /**
   * Focus and highlight text inside link input when shown
   */
  $effect(() => {
    if (showLinkInput && inputElement) {
      inputElement.focus();
      inputElement.select();
    }
  });

  /**
   * Handles clicking outside the toolbar to close it
   */
  $effect(() => {
    if (!visible) return;

    const handleOutsideClick = (e: MouseEvent) => {
      if (container && !container.contains(e.target as globalThis.Node)) {
        visible = false;
        showLinkInput = false;
      }
    };

    document.addEventListener('mousedown', handleOutsideClick, true);
    return () => {
      document.removeEventListener('mousedown', handleOutsideClick, true);
    };
  });

  /**
   * Listens to selection changes and mouse/key events to control toolbar visibility.
   */
  $effect(() => {
    document.addEventListener('selectionchange', handleSelectionChange);
    window.addEventListener('resize', handleSelectionChange);
    document.addEventListener('scroll', handleSelectionChange, true);

    // Track selection start
    const handleMouseDown = () => {
      isSelecting = true;
      isMakingNewSelection = true;
    };

    // Track selection end
    const handleMouseUp = () => {
      isSelecting = false;

      if (isMakingNewSelection && !shouldPreventToolbarReopen)
        showPendingToolbar();

      isMakingNewSelection = false;

      // Reset prevention flag after mouseup
      if (shouldPreventToolbarReopen) {
        setTimeout(() => {
          shouldPreventToolbarReopen = false;
        }, 50);
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      // Track when keyboard selection starts (Shift + Arrow keys)
      if (
        e.shiftKey &&
        ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(e.key)
      ) {
        isKeyboardSelecting = true;
        isSelecting = true;
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      // Show toolbar only after Shift key is released (end of keyboard selection)
      if (e.key === 'Shift' && isKeyboardSelecting) {
        isKeyboardSelecting = false;
        isSelecting = false;
        const selection = window.getSelection();
        if (selection && !selection.isCollapsed) {
          setTimeout(() => {
            showPendingToolbar();
          }, 10);
        }
      }
    };

    document.addEventListener('mousedown', handleMouseDown);
    document.addEventListener('mouseup', handleMouseUp);
    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);

    return () => {
      document.removeEventListener('selectionchange', handleSelectionChange);
      window.removeEventListener('resize', handleSelectionChange);
      document.removeEventListener('scroll', handleSelectionChange, true);
      document.removeEventListener('mousedown', handleMouseDown);
      document.removeEventListener('mouseup', handleMouseUp);
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('keyup', handleKeyUp);
    };
  });
</script>

{#if visible}
  <div
    bind:this={container}
    class="floating-toolbar"
    style="--toolbar-x: {x}px; --toolbar-y: {y}px;"
    role="toolbar"
    aria-label="Text formatting"
    tabindex="-1"
    onmousedown={(e) => e.preventDefault()}
    in:fly={{ y: 20, duration: 150 }}
    out:fly={{ y: -20, duration: 200 }}
  >
    {#if showLinkInput}
      <form
        class="link-form"
        onsubmit={(e) => {
          e.preventDefault();
          applyLink(linkUrl);
        }}
      >
        <input
          bind:this={inputElement}
          type="text"
          placeholder="Paste or type URL..."
          bind:value={linkUrl}
          class="link-input"
          onkeydown={(e) => {
            if (e.key === 'Escape') {
              showLinkInput = false;
            }
          }}
          onmousedown={(e) => e.stopPropagation()}
        />

        <button type="submit" class="toolbar-btn active" title="Apply Link">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="1.1rem"
            viewBox="0 -960 960 960"
            width="1.1rem"
            fill="currentColor"
          >
            <path d="M382-240 154-468l57-57 171 171 367-367 57 57-424 424Z" />
          </svg>
        </button>

        {#if isLink}
          <button
            type="button"
            class="toolbar-btn danger"
            onclick={() => applyLink('')}
            title="Remove Link"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              height="1.1rem"
              viewBox="0 -960 960 960"
              width="1.1rem"
              fill="currentColor"
            >
              <path d="M280-440v-80h400v80H280Z" />
            </svg>
          </button>
        {/if}

        <button
          type="button"
          class="toolbar-btn"
          onclick={() => {
            showLinkInput = false;
          }}
          title="Cancel"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="1.1rem"
            viewBox="0 -960 960 960"
            width="1.1rem"
            fill="currentColor"
          >
            <path
              d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z"
            />
          </svg>
        </button>
      </form>
    {:else}
      <button
        class="toolbar-btn"
        class:active={isBold}
        onclick={() => toggleFormat('strong')}
        title="Bold"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
          ><path
            d="M272-200v-560h221q65 0 120 40t55 111q0 51-23 78.5T602-491q25 11 55.5 41t30.5 90q0 89-65 124.5T501-200H272Zm121-112h104q48 0 58.5-24.5T566-372q0-11-10.5-35.5T494-432H393v120Zm0-228h93q33 0 48-17t15-38q0-24-17-39t-44-15h-95v109Z"
          /></svg
        >
      </button>

      <button
        class="toolbar-btn"
        class:active={isItalic}
        onclick={() => toggleFormat('emphasis')}
        title="Italic"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
          ><path
            d="M200-200v-100h160l120-360H320v-100h400v100H580L460-300h140v100H200Z"
          /></svg
        >
      </button>

      <button
        class="toolbar-btn"
        class:active={isStrikethrough}
        onclick={() => toggleFormat('strike_through')}
        title="Strikethrough"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
          ><path
            d="M80-400v-80h800v80H80Zm340-160v-120H200v-120h560v120H540v120H420Zm0 400v-160h120v160H420Z"
          /></svg
        >
      </button>

      <button
        class="toolbar-btn"
        class:active={isCode}
        onclick={() => toggleFormat('inlineCode')}
        title="Code"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
          ><path
            d="M320-240 80-480l240-240 57 57-184 184 183 183-56 56Zm320 0-57-57 184-184-183-183 56-56 240 240-240 240Z"
          /></svg
        >
      </button>

      <button
        class="toolbar-btn"
        class:active={isLink}
        onclick={() => {
          showLinkInput = true;
        }}
        title="Link"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="1.1rem"
          viewBox="0 -960 960 960"
          width="1.1rem"
          fill="currentColor"
          ><path
            d="M318-120q-82 0-140-58t-58-140q0-40 15-76t43-64l134-133 56 56-134 134q-17 17-25.5 38.5T200-318q0 49 34.5 83.5T318-200q23 0 45-8.5t39-25.5l133-134 57 57-134 133q-28 28-64 43t-76 15Zm79-220-57-57 223-223 57 57-223 223Zm251-28-56-57 134-133q17-17 25-38t8-44q0-50-34-85t-84-35q-23 0-44.5 8.5T558-726L425-592l-57-56 134-134q28-28 64-43t76-15q82 0 139.5 58T839-641q0 39-14.5 75T782-502L648-368Z"
          /></svg
        >
      </button>
    {/if}
  </div>
{/if}

<style>
  .floating-toolbar {
    position: fixed;
    z-index: 9999;
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem;
    background: var(--bg-surface, var(--bg-base));
    border-radius: 0.25rem;
    box-shadow:
      0 6px 16px rgba(0, 0, 0, 0.12),
      0 2px 4px rgba(0, 0, 0, 0.06);
    backdrop-filter: blur(8px);
    pointer-events: auto;
    max-width: 90vw;
    overflow: hidden;
    left: var(--toolbar-x);
    top: var(--toolbar-y);
    transform: translateX(-50%) translateY(-100%);
    margin: 0;
    transition:
      left 0.1s ease,
      top 0.1s ease;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.85rem;
    height: 1.85rem;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-ui-muted, var(--text-muted));
    cursor: pointer;
    transition:
      background-color 0.15s,
      color 0.15s;
    padding: 0;
  }

  .toolbar-btn:hover {
    background-color: color-mix(in srgb, var(--accent) 15%, transparent);
    color: var(--text-main);
  }

  .toolbar-btn.active {
    background-color: color-mix(in srgb, var(--accent) 20%, transparent);
    color: var(--accent);
  }

  .toolbar-btn.danger:hover {
    background-color: color-mix(in srgb, #ef596f 15%, transparent);
    color: #ef596f;
  }

  .link-form {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .link-input {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.25rem 0.5rem;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.85rem;
    width: 12rem;
    outline: none;
    transition: border-color 0.15s;
  }

  .link-input:focus {
    border-color: var(--accent);
  }
</style>
