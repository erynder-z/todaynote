# Adding keyboard shortcuts to TodayNote

This guide explains how to add new keyboard shortcuts to TodayNote, covering the backend configuration, frontend display, and input handling.

## Table of Contents

- [Overview](#overview)
- [Step-by-Step Guide](#step-by-step-guide)
  - [1. Add the shortcut to backend config](#1-add-the-shortcut-to-backend-config)
  - [2. Add the shortcut to frontend display](#2-add-the-shortcut-to-frontend-display)
  - [3. Understand the input handling system](#3-understand-the-input-handling-system)
  - [4. Use the shortcut in components](#4-use-the-shortcut-in-components)
- [Shortcut Configuration Options](#shortcut-configuration-options)
- [Best Practices](#best-practices)
- [Example: Adding a New Shortcut](#example-adding-a-new-shortcut)
- [Troubleshooting](#troubleshooting)

## Overview

TodayNote's keyboard shortcut system consists of three main parts:

1. **Backend Configuration** (`src-tauri/src/models/config.rs`) - Defines default shortcuts and persists user customizations
2. **Frontend Display** (`src/lib/components/ShortcutListModal.svelte`) - Shows available shortcuts in the UI
3. **Input Handling** (`src/lib/stores/input.svelte.ts`) - Detects key presses and triggers actions

## Step-by-Step Guide

### 1. Add the shortcut to backend config

**File:** `src-tauri/src/models/config.rs`

Add the new shortcut to the `shortcuts` HashMap in the `Default` implementation:

```rust
shortcuts.insert(
    "yourActionName".to_string(),
    ShortcutConfig {
        key: "key".to_string(),          // The key to press (e.g., "n", "Escape", "F1")
        primary: true,                   // Whether primary modifier is required (⌘/Ctrl)
        secondary: false,                 // Whether secondary modifier is required (⌥/Shift)
        shift: false,                    // Whether Shift key is required (independent of secondary)
        description: "Action description".to_string(), // User-friendly description
    },
);
```

**Key considerations:**
- Use a unique action name that describes the functionality 
- Choose keys that don't conflict with existing shortcuts
- Provide a clear, concise description for the user

### 2. Add the shortcut to frontend display

**File:** `src/lib/components/ShortcutListModal.svelte`

Add action name to the appropriate section:

```typescript
// For global shortcuts (i.e. actions that can be triggered from anywhere)
const globalActions: ShortcutAction[] = [
    // ... existing actions
    'actionName',
];

// For categorized shortcuts (shortcuts that only work in specific locations, such as search, thread, etc.)
const yourCategoryShortcut = settings.shortcuts.yourActionName;
```

Add display section:

```svelte
<div class="category-shortcuts">
    <h3>{$t('shortcuts.category.description')}</h3>
    {#if categoryShortcut}
        <div class="shortcut-item">
            <span class="shortcut-description">{$t('shortcuts.category.your_action')}</span>
            <div class="shortcut-keys">
                <KeyboardShortcut
                    primary={categoryShortcut.primary}
                    secondary={categoryShortcut.secondary}
                    key={categoryShortcut.key.toUpperCase()}
                />
            </div>
        </div>
    {/if}
</div>
```

Add translation keys to localization files:

```json
{
    "shortcuts.category.description": "Some shortcut",
    "shortcuts.category.your_action": "Action description"
}
```

### 3. Understanding the input handling system

**File:** `src/lib/stores/input.svelte.ts`

The `InputManager` class handles keyboard input detection and shortcut triggering:

**Key concepts:**
- **Primary modifier**: ⌘ (Mac) or Ctrl (Windows/Linux)
- **Secondary modifier**: ⌥ (Mac) or Shift (Windows/Linux)
- **Modifier detection**: The system automatically handles platform differences
- **Input context**: Shortcuts are disabled in text inputs unless they use modifiers

**How shortcut matching works:**
1. User presses a key combination
2. `handleKeyDown` checks all registered shortcuts
3. For each shortcut, it verifies:
   - Key matches (handles multiple key formats)
   - Required modifiers are pressed
   - Unspecified modifiers are NOT pressed
4. If matched, the callback is executed

### 4. Use the shortcut in components

In the component, register the shortcut using the `useShortcuts` hook:

```svelte
<script lang="ts">
import { useShortcuts } from '$lib/utils/shortcuts';

const handleYourAction = (e: KeyboardEvent) => {
    // Action logic here
    console.log('Your action triggered!');
    return true; // Return false to allow event propagation
};

useShortcuts({
    yourActionName: handleYourAction,
});
</script>
```

**Callback return values:**
- `return true` - Prevents default behavior and stops event propagation
- `return false` - Allows default behavior and event propagation

## Shortcut Configuration Options

### Key Formats Supported

The system supports multiple key format specifications:

```rust
key: "k".to_string(),          // Single character
key: "Escape".to_string(),    // Special key name
key: "1,2,3".to_string(),     // Multiple keys (any one triggers)
key: "a,b,c,d".to_string(),    // Multiple keys with comma separation
```

### Modifier Combinations

```rust
// No modifiers
ShortcutConfig {
    key: "k".to_string(),
    primary: false,
    secondary: false,
    shift: false,
    description: "Simple key press".to_string(),
}

// Primary modifier only (⌘/Ctrl + K)
ShortcutConfig {
    key: "k".to_string(),
    primary: true,
    secondary: false,
    shift: false,
    description: "Primary + Key".to_string(),
}

// Primary + Secondary (⌘+⌥/Ctrl+Shift + K)
ShortcutConfig {
    key: "k".to_string(),
    primary: true,
    secondary: true,
    shift: false,
    description: "Primary + Secondary + Key".to_string(),
}

// With Shift (⌘+Shift/K or Ctrl+Shift+K)
ShortcutConfig {
    key: "k".to_string(),
    primary: true,
    secondary: false,
    shift: true,
    description: "Primary + Shift + Key".to_string(),
}
```

### Special Keys

The system handles special keys automatically:
- `Escape`, `Enter`, `Tab`, `Space`
- `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`
- `PageUp`, `PageDown`, `Home`, `End`

## Guidelines

### Key Selection

1. **Avoid conflicts** with browser/system shortcuts
2. **Use mnemonic keys** when possible (e.g., `n` for "new", `s` for "save")

### Modifier Usage

1. **Primary modifier (⌘/Ctrl)** - For main actions
2. **Primary + Secondary (⌘+⌥/Ctrl+Shift)** - For secondary actions
3. **Shift alone** - Currently not used
4. **No modifiers** - Only for non-typing contexts or with explicit user intent


## Example: Adding a New Shortcut

Add a shortcut to toggle a new feature called "Zen Mode":

### 1. Backend Configuration

**File:** `src-tauri/src/models/config.rs`

```rust
shortcuts.insert(
    "toggleZenMode".to_string(),
    ShortcutConfig {
        key: "z".to_string(),
        primary: true,
        secondary: true,  // ⌘+⌥+Z or Ctrl+Shift+Z
        shift: false,
        description: "Toggle Zen Mode".to_string(),
    },
);
```

### 2. Frontend Display

**File:** `src/lib/components/ShortcutListModal.svelte`

```typescript
// Add to global actions
const globalActions: ShortcutAction[] = [
    // ... existing actions
    'toggleZenMode',
];
```

### 3. Component Usage

**File:** `src/lib/components/ZenModeToggle.svelte`

```svelte
<script lang="ts">
import { useShortcuts } from '$lib/utils/shortcuts';
import { sessionState } from '$lib/stores/sessionState.svelte';

const toggleZenMode = () => {
    sessionState.zenModeEnabled = !sessionState.zenModeEnabled;
    return true; // Prevent default behavior
};

useShortcuts({
    toggleZenMode,
});
</script>
```

### 4. Add to Session State (if needed)

**File:** `src/lib/interfaces/appState.ts`

```typescript
export interface SessionState {
    // ... existing fields
    zenModeEnabled: boolean;
}
```

**File:** `src/lib/stores/sessionState.svelte.ts`

```typescript
export const sessionState = $state<SessionState>({
    // ... existing defaults
    zenModeEnabled: false,
});
```

## Files Involved in Shortcut System

### Backend Files:
- `src-tauri/src/models/config.rs` - Shortcut definitions and defaults
- `src-tauri/src/models/response_types.rs` - Shortcut response types
- `src-tauri/src/commands/settings.rs` - Shortcut saving/loading commands

### Frontend Files:
- `src/lib/components/ShortcutListModal.svelte` - Shortcut display UI
- `src/lib/stores/input.svelte.ts` - Input detection and handling
- `src/lib/utils/shortcuts.ts` - Shortcut registration utilities
- `src/lib/interfaces/input.ts` - Shortcut type definitions
- `src/lib/types/input.ts` - Shortcut action types

### Key Interfaces:
- `ShortcutConfig` - Individual shortcut configuration
- `ShortcutAction` - Type-safe action names
- `ShortcutCallback` - Callback function type
