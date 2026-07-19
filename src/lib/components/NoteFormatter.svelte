<script lang="ts">
  import type { Editor } from '@milkdown/core';
  import { editorViewCtx } from '@milkdown/core';
  import type { Ctx } from '@milkdown/ctx';
  import { toggleMark } from '@milkdown/prose/commands';
  import type { Mark, Node as PMNode } from 'prosemirror-model';
  import type { EditorState } from 'prosemirror-state';

  let { editorInstance } = $props<{ editorInstance: Editor | null }>();

  let isBold = $state(false);
  let isItalic = $state(false);
  let isStrikethrough = $state(false);
  let isCode = $state(false);
  let isLink = $state(false);

  let showLinkInput = $state(false);
  let linkUrl = $state('');
  let inputElement = $state<HTMLInputElement | null>(null);

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
   * Updates state of marks in the current selection
   */
  const updateSelectionState = () => {
    if (!editorInstance) return;
    try {
      editorInstance.action((ctx: Ctx) => {
        const view = ctx.get(editorViewCtx);
        const { state } = view;
        const { from, to, $from: fromPos } = state.selection;

        const strong = state.schema.marks.strong;
        const emphasis = state.schema.marks.emphasis;
        const strike_through = state.schema.marks.strike_through;
        const inlineCode = state.schema.marks.inlineCode;
        const link = state.schema.marks.link;

        isBold = strong
          ? state.doc.rangeHasMark(from, to, strong) ||
            fromPos.marks().some((m: Mark) => m.type.name === 'strong')
          : false;
        isItalic = emphasis
          ? state.doc.rangeHasMark(from, to, emphasis) ||
            fromPos.marks().some((m: Mark) => m.type.name === 'emphasis')
          : false;
        isStrikethrough = strike_through
          ? state.doc.rangeHasMark(from, to, strike_through) ||
            fromPos.marks().some((m: Mark) => m.type.name === 'strike_through')
          : false;
        isCode = inlineCode
          ? state.doc.rangeHasMark(from, to, inlineCode) ||
            fromPos.marks().some((m: Mark) => m.type.name === 'inlineCode')
          : false;
        isLink = link
          ? state.doc.rangeHasMark(from, to, link) ||
            fromPos.marks().some((m: Mark) => m.type.name === 'link')
          : false;

        // Extract active link URL if we're not currently editing it
        if (!showLinkInput) {
          const linkMark = link
            ? fromPos.marks().find((m: Mark) => m.type.name === 'link') ||
              (from !== to && state.doc.rangeHasMark(from, to, link)
                ? findLinkMarkInRange(state, from, to)
                : null)
            : null;
          linkUrl = linkMark ? linkMark.attrs.href || '' : '';
        }
      });
    } catch (e) {
      console.error('Error updating selection state', e);
    }
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
      view.focus();
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
      view.focus();
    });

    showLinkInput = false;
    updateSelectionState();
  };

  /**
   * Cancels the link entry and returns focus to the editor
   */
  const cancelLink = () => {
    showLinkInput = false;
    if (editorInstance) {
      editorInstance.action((ctx: Ctx) => {
        const view = ctx.get(editorViewCtx);
        view.focus();
      });
    }
  };

  // Focus and highlight text inside link input when shown
  $effect(() => {
    if (showLinkInput && inputElement) {
      inputElement.focus();
      inputElement.select();
    }
  });

  // Pull initial state when editorInstance is set
  $effect(() => {
    if (editorInstance) updateSelectionState();
  });

  // Listen to window level selection changes to keep format highlights updated while visible
  $effect(() => {
    const handleSelectionUpdate = () => {
      const selection = window.getSelection();
      if (selection && !selection.isCollapsed) {
        updateSelectionState();
      }
    };
    document.addEventListener('selectionchange', handleSelectionUpdate);
    return () =>
      document.removeEventListener('selectionchange', handleSelectionUpdate);
  });
</script>

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
          e.preventDefault();
          cancelLink();
        }
      }}
      onmousedown={(e) => e.stopPropagation()}
    />

    <button
      type="submit"
      class="toolbar-btn active"
      onmousedown={(e) => e.preventDefault()}
      title="Apply Link"
    >
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
        onmousedown={(e) => e.preventDefault()}
        onclick={(e) => {
          e.stopPropagation();
          applyLink('');
        }}
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
      onmousedown={(e) => e.preventDefault()}
      onclick={(e) => {
        e.stopPropagation();
        cancelLink();
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
    onmousedown={(e) => e.preventDefault()}
    onclick={(e) => {
      e.stopPropagation();
      toggleFormat('strong');
    }}
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
    onmousedown={(e) => e.preventDefault()}
    onclick={(e) => {
      e.stopPropagation();
      toggleFormat('emphasis');
    }}
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
    onmousedown={(e) => e.preventDefault()}
    onclick={(e) => {
      e.stopPropagation();
      toggleFormat('strike_through');
    }}
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
    onmousedown={(e) => e.preventDefault()}
    onclick={(e) => {
      e.stopPropagation();
      toggleFormat('inlineCode');
    }}
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
    onmousedown={(e) => e.preventDefault()}
    onclick={(e) => {
      e.stopPropagation();
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

<style>
  .toolbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-ui-muted, var(--text-muted));
    cursor: pointer;
    transition:
      background-color 0.2s cubic-bezier(0.4, 0, 0.2, 1),
      color 0.2s,
      transform 0.1s ease;
    padding: 0;
  }

  .toolbar-btn:hover {
    background-color: color-mix(in srgb, var(--accent) 12%, transparent);
    color: var(--accent);
    transform: translateY(-1px);
  }

  .toolbar-btn:active {
    transform: scale(0.92);
  }

  .toolbar-btn.active {
    background-color: color-mix(in srgb, var(--accent) 18%, transparent);
    color: var(--accent);
  }

  .toolbar-btn.danger:hover {
    background-color: color-mix(
      in srgb,
      var(--error, #ef596f) 15%,
      transparent
    );
    color: var(--error, #ef596f);
  }

  .link-form {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .link-input {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.3rem 0.6rem;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.85rem;
    width: 13rem;
    outline: none;
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }

  .link-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 20%, transparent);
  }
</style>
