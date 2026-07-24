<script lang="ts">
  import { type Snippet, tick } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  let {
    children,
    editorWrapperClass = 'milkdown-editor-wrapper',
    linkInputActive = false,
  } = $props<{
    children: Snippet<[boolean]>;
    editorWrapperClass?: string;
    linkInputActive?: boolean;
  }>();

  let visible = $state(false);
  let x = $state(0);
  let y = $state(0);
  let positionBelow = $state(false);
  let container: HTMLDivElement | null = $state(null);

  let isSelecting = $state(false);
  let isKeyboardSelecting = $state(false);

  /**
   * Checks if a DOM node is inside the editor wrapper.
   */
  const isNodeInEditor = (node: Node | null): boolean => {
    let curr = node;
    while (curr) {
      if (
        curr instanceof HTMLElement &&
        curr.classList.contains(editorWrapperClass)
      )
        return true;

      curr = curr.parentNode;
    }
    return false;
  };

  /**
   * Checks if a DOM node is inside a thread marker.
   */
  const isNodeInThreadMarker = (node: Node | null): boolean => {
    let curr = node;
    while (curr) {
      if (
        curr instanceof HTMLElement &&
        curr.classList.contains('thread-marker')
      )
        return true;

      curr = curr.parentNode;
    }
    return false;
  };

  /**
   * Checks if the selection is inside the Milkdown editor.
   */
  const isSelectionInEditor = (selection: Selection | null): boolean => {
    return !!selection?.anchorNode && isNodeInEditor(selection.anchorNode);
  };

  /**
   * Checks if the selection is inside a thread marker.
   */
  const isSelectionInThreadMarker = (selection: Selection | null): boolean => {
    return (
      !!selection?.anchorNode && isNodeInThreadMarker(selection.anchorNode)
    );
  };

  /**
   * Checks if the current selection is valid (non-empty and not collapsed).
   */
  const isSelectionValid = (selection: Selection | null): boolean => {
    if (!selection) return false;
    return !selection.isCollapsed && !!selection.toString().trim();
  };

  /**
   * Positions the toolbar above or below the current selection.
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

    // Use measured container dimensions if available, otherwise fall back to defaults
    const width = container ? container.offsetWidth : 250;
    const height = container ? container.offsetHeight : 40;

    // Center horizontally above selection
    const idealX = rect.left + rect.width / 2;

    const safeMargin = 12;
    const minX = width / 2 + safeMargin;
    const maxX = viewportWidth - width / 2 - safeMargin;

    if (maxX < minX) {
      x = viewportWidth / 2;
    } else {
      x = Math.max(minX, Math.min(idealX, maxX));
    }

    // Determine vertical placement: prefer above selection, fallback below
    const spacing = 8;
    const spaceAbove = rect.top - safeMargin;
    const spaceNeeded = height + spacing;

    if (spaceAbove < spaceNeeded) {
      // Place below
      y = rect.bottom + spacing;
      positionBelow = true;
    } else {
      // Place above
      y = rect.top - spacing;
      positionBelow = false;
    }

    // Hide if selection rect itself is completely scrolled out of the viewport
    if (rect.bottom < 0 || rect.top > viewportHeight) visible = false;
  };

  /**
   * Handles selection changes to prepare toolbar positioning but delays showing it.
   */
  const handleSelectionChange = () => {
    // Show/hide floating toolbar based on selection state and link input activity
    const selection = window.getSelection();

    if (!selection || isSelectionInThreadMarker(selection)) {
      visible = false;
      return;
    }

    if (isSelecting) {
      visible = false;
      return;
    }

    // Check if selection is in editor or if link input is active
    const selectionInEditor = isSelectionInEditor(selection);
    const selectionValid = isSelectionValid(selection);

    // Case 1: Selection is valid and in editor - show toolbar normally
    if (selectionValid && selectionInEditor) {
      positionToolbar(selection);
      visible = true;
      return;
    }

    // Case 2: Link input is active - keep toolbar visible regardless of selection
    if (linkInputActive) {
      if (selectionInEditor) {
        positionToolbar(selection);
      }
      visible = true;
      return;
    }

    // Case 3: Selection is collapsed or not in editor, and link input is not active
    visible = false;
  };

  /**
   * Highly efficient repositioning handler for scroll and resize events.
   */
  const handleScrollOrResize = () => {
    if (!visible) return;
    const selection = window.getSelection();
    if (selection && isSelectionValid(selection)) {
      positionToolbar(selection);
    }
  };

  /**
   * Shows the toolbar with the pending selection after mouse/key release
   */
  const showPendingToolbar = () => {
    if (isSelecting) return;
    const selection = window.getSelection();
    if (
      selection &&
      isSelectionValid(selection) &&
      isSelectionInEditor(selection) &&
      !isSelectionInThreadMarker(selection)
    ) {
      positionToolbar(selection);
      visible = true;
    }
  };

  // Recalculate layout metrics on mount/re-position after container becomes available in DOM
  $effect(() => {
    if (visible && container) {
      const selection = window.getSelection();
      if (selection && !selection.isCollapsed) {
        positionToolbar(selection);
      }
    }
  });

  // Watch for changes in container dimensions (e.g. switching between buttons and form)
  $effect(() => {
    if (visible && container) {
      const el = container;
      const observer = new ResizeObserver(() => {
        const newWidth = el.offsetWidth;
        const viewportWidth = window.innerWidth;
        const safeMargin = 12;
        const minX = newWidth / 2 + safeMargin;
        const maxX = viewportWidth - newWidth / 2 - safeMargin;
        x = Math.max(minX, Math.min(x, maxX));
      });
      observer.observe(el);
      return () => observer.disconnect();
    }
  });

  // Register selection and interaction event listeners
  $effect(() => {
    document.addEventListener('selectionchange', handleSelectionChange);
    window.addEventListener('resize', handleScrollOrResize);
    document.addEventListener('scroll', handleScrollOrResize, true);

    const handleMouseDown = (e: MouseEvent) => {
      isSelecting = true;

      // Close toolbar if clicking outside both the toolbar and editor
      if (visible && container && !container.contains(e.target as Node)) {
        if (!isNodeInEditor(e.target as Node)) visible = false;
      }
    };

    const handleMouseUp = () => {
      isSelecting = false;
      showPendingToolbar();
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && visible) {
        e.preventDefault();
        visible = false;
      }
      if (
        e.shiftKey &&
        ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(e.key)
      ) {
        isKeyboardSelecting = true;
        isSelecting = true;
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.key === 'Shift' && isKeyboardSelecting) {
        isKeyboardSelecting = false;
        isSelecting = false;
        showPendingToolbar();
      }
    };

    window.addEventListener('mousedown', handleMouseDown);
    window.addEventListener('mouseup', handleMouseUp);
    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);

    return () => {
      document.removeEventListener('selectionchange', handleSelectionChange);
      window.removeEventListener('resize', handleScrollOrResize);
      document.removeEventListener('scroll', handleScrollOrResize, true);
      window.removeEventListener('mousedown', handleMouseDown);
      window.removeEventListener('mouseup', handleMouseUp);
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('keyup', handleKeyUp);
    };
  });
</script>

{#if visible}
  <div
    class="floating-toolbar-wrapper"
    style="--toolbar-x: {x}px; --toolbar-y: {y}px; --toolbar-translate-y: {positionBelow
      ? '0%'
      : '-100%'};"
  >
    <div
      bind:this={container}
      class="floating-toolbar"
      role="toolbar"
      aria-label="Selection options"
      tabindex="-1"
      onmousedown={(e) => e.stopPropagation()}
      in:scale={{ duration: 120, start: 0.95 }}
      out:fade={{ duration: 80 }}
    >
      {@render children(positionBelow)}
    </div>
  </div>
{/if}

<style>
  .floating-toolbar-wrapper {
    position: fixed;
    z-index: 9999;
    left: 0;
    top: 0;
    transform: translate(var(--toolbar-x), var(--toolbar-y)) translateX(-50%)
      translateY(var(--toolbar-translate-y, -100%));
    pointer-events: none;
    margin: 0;
  }

  .floating-toolbar {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem;
    background: var(--bg-surface, var(--bg-base));
    border-radius: 0.25rem;
    box-shadow:
      0 10px 30px -5px rgba(0, 0, 0, 0.2),
      0 4px 12px -4px rgba(0, 0, 0, 0.1);
    backdrop-filter: blur(12px) saturate(140%);
    pointer-events: auto;
    max-width: 90vw;
    overflow: hidden;
  }
</style>
