<script lang="ts">
  import type { Editor } from '@milkdown/core';
  import { editorViewCtx } from '@milkdown/core';
  import type { Ctx } from '@milkdown/ctx';
  import { lift, toggleMark, wrapIn } from '@milkdown/prose/commands';
  import type { Mark, Node as PMNode } from 'prosemirror-model';
  import type { EditorState } from 'prosemirror-state';
  import { t } from '$lib/utils/i18n';

  let { editorInstance } = $props<{ editorInstance: Editor | null }>();

  let isBold = $state(false);
  let isItalic = $state(false);
  let isStrikethrough = $state(false);
  let isCode = $state(false);
  let isQuote = $state(false);
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
        const codeBlock = state.schema.nodes.code_block;
        const blockquote = state.schema.nodes.blockquote;
        const isInInlineCode = inlineCode
          ? state.doc.rangeHasMark(from, to, inlineCode) ||
            fromPos.marks().some((m: Mark) => m.type.name === 'inlineCode')
          : false;
        const isInCodeBlock = codeBlock
          ? fromPos.parent.type === codeBlock
          : false;
        isCode = isInInlineCode || isInCodeBlock;
        isQuote = blockquote ? isInBlockquote(state) : false;
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
   * Toggles code formatting: inline code for single line selection, code block for multi-line selection.
   */
  const toggleCode = () => {
    if (!editorInstance) return;

    editorInstance.action((ctx: Ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state, dispatch } = view;
      const { from, to, $from: fromPos } = state.selection;

      const codeBlockType = state.schema.nodes.code_block;
      const inlineCodeType = state.schema.marks.inlineCode;
      const paragraphType = state.schema.nodes.paragraph;

      const isInCodeBlock =
        codeBlockType && fromPos.parent.type === codeBlockType;

      if (isInCodeBlock) {
        if (paragraphType) {
          const codeBlockNode = fromPos.parent;
          const textContent = codeBlockNode.textContent;
          const lines = textContent.split('\n');
          const paragraphNodes = lines.map((line) => {
            const textNode = line ? state.schema.text(line) : null;
            return paragraphType.create(null, textNode);
          });
          const startPos = fromPos.before();
          const endPos = fromPos.after();
          const tr = state.tr.replaceWith(startPos, endPos, paragraphNodes);
          dispatch(tr);
        }
      } else {
        const text = state.doc.textBetween(from, to, '\n');
        const isMultiLine =
          text.includes('\n') || fromPos.parent !== state.selection.$to.parent;

        if (isMultiLine && codeBlockType) {
          const textNode = text ? state.schema.text(text) : null;
          const newCodeBlockNode = codeBlockType.create(null, textNode);
          const tr = state.tr.replaceSelectionWith(newCodeBlockNode);
          dispatch(tr);
        } else if (inlineCodeType) {
          toggleMark(inlineCodeType)(state, dispatch);
        }
      }

      view.focus();
      updateSelectionState();
    });
  };

  /**
   * Checks if the selection is inside a blockquote
   */
  const isInBlockquote = (state: EditorState): boolean => {
    const { $from: fromPos } = state.selection;
    const blockquoteType = state.schema.nodes.blockquote;
    if (!blockquoteType) return false;

    for (let depth = fromPos.depth; depth > 0; depth--) {
      if (fromPos.node(depth).type === blockquoteType) {
        return true;
      }
    }
    return false;
  };

  /**
   * Toggles wrapping selection in a blockquote
   */
  const toggleBlockquote = () => {
    if (!editorInstance) return;

    editorInstance.action((ctx: Ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state, dispatch } = view;
      const blockquoteType = state.schema.nodes.blockquote;
      if (!blockquoteType) return;

      const inQuote = isInBlockquote(state);

      if (inQuote) {
        lift(state, dispatch);
      } else {
        wrapIn(blockquoteType)(state, dispatch);
      }

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

  /**
   * Copies the selected text to the clipboard (Plain text)
   */
  const copyToClipboard = async () => {
    if (!editorInstance) return;

    editorInstance.action(async (ctx: Ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state } = view;
      const { from, to } = state.selection;

      if (from === to) return;

      const text = state.doc.textBetween(from, to, '\n');
      await navigator.clipboard.writeText(text);
    });
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
      placeholder={$t('note_formatter.link.placeholder')}
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
      title={$t('note_formatter.link.apply')}
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
        title={$t('note_formatter.link.remove')}
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
      title={$t('note_formatter.link.cancel')}
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
    title={$t('note_formatter.bold')}
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
    title={$t('note_formatter.italic')}
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
    title={$t('note_formatter.strikethrough')}
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
      toggleCode();
    }}
    title={$t('note_formatter.code')}
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
    class:active={isQuote}
    onmousedown={(e) => e.preventDefault()}
    onclick={(e) => {
      e.stopPropagation();
      toggleBlockquote();
    }}
    title={$t('note_formatter.quote')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1.1rem"
      viewBox="0 -960 960 960"
      width="1.1rem"
      fill="currentColor"
      ><path
        d="m228-240 92-160q-66 0-113-47t-47-113q0-66 47-113t113-47q66 0 113 47t47 113q0 23-5.5 42.5T458-480L320-240h-92Zm360 0 92-160q-66 0-113-47t-47-113q0-66 47-113t113-47q66 0 113 47t47 113q0 23-5.5 42.5T818-480L680-240h-92ZM362.5-517.5Q380-535 380-560t-17.5-42.5Q345-620 320-620t-42.5 17.5Q260-585 260-560t17.5 42.5Q295-500 320-500t42.5-17.5Zm360 0Q740-535 740-560t-17.5-42.5Q705-620 680-620t-42.5 17.5Q620-585 620-560t17.5 42.5Q655-500 680-500t42.5-17.5ZM680-560Zm-360 0Z"
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
    title={$t('note_formatter.link')}
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
  <button
    class="toolbar-btn"
    onmousedown={(e) => e.preventDefault()}
    onclick={(e) => {
      e.stopPropagation();
      copyToClipboard();
    }}
    title={$t('note_formatter.copy')}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="1.1rem"
      viewBox="0 -960 960 960"
      width="1.1rem"
      fill="currentColor"
      ><path
        d="M360-240q-33 0-56.5-23.5T280-320v-480q0-33 23.5-56.5T360-880h360q33 0 56.5 23.5T800-800v480q0 33-23.5 56.5T720-240H360Zm0-80h360v-480H360v480ZM200-80q-33 0-56.5-23.5T120-160v-560h80v560h440v80H200Zm160-240v-480 480Z"
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
    border: none;
    border-radius: 0.25rem;
    padding: 0.3rem 0.6rem;
    color: var(--text-main);
    font-family: inherit;
    font-size: 0.85rem;
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
