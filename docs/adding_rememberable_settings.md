# Adding a new setting to TodayNote

This guide explains how the settings system works and how to add a new setting to TodayNote that can be persisted when "Remember Component Settings" is enabled.

## Table of Contents

- [Overview](#overview)
- [How the Settings System Works](#how-the-settings-system-works)
- [Step-by-Step Guide](#step-by-step-guide)
  - [1. Add the setting to the backend config](#1-add-the-setting-to-the-backend-config)
  - [2. Add the setting to the frontend interfaces](#2-add-the-setting-to-the-frontend-interfaces)
  - [3. Add the setting to the response types](#3-add-the-setting-to-the-response-types)
  - [4. Add the setting to the SettingsStore](#4-add-the-setting-to-the-settingsstore)
  - [5. Update the app setup synchronization](#5-update-the-app-setup-synchronization)
  - [6. Update the backend commands](#6-update-the-backend-commands)
  - [7. Add the setting to setup commands](#7-add-the-setting-to-setup-commands)
  - [8. Create a method to save the setting (optional)](#8-create-a-method-to-save-the-setting-optional)
  - [9. Use the setting in components](#9-use-the-setting-in-components)
- [Example: Adding Thread Shortcuts Mode](#example-adding-thread-shortcuts-mode)
- [Best Practices](#best-practices)
- [Testing](#testing)

## Overview

TodayNote has two types of settings:
1. **Critical settings** - Always remembered (theme, locale, notes folder)
2. **Component settings** - Remembered only when "Remember Component Settings" is enabled (search mode, sidebar state, thread mode, etc.)

## How the Settings System Works

### Architecture Overview

```
┌───────────────────────────────────────────────────────────────┐
│                        Frontend (Svelte)                      │
├─────────────────┬─────────────────┬─────────────────┬────────────┤
│  SettingsStore   │  SessionState   │    Components   │  Input    │
│  (Persistent)    │  (Runtime)      │  (UI Elements)  │  (Shortcuts)│
└─────────────────┴─────────────────┴─────────────────┴────────────┘
                            │
                            ▼
┌───────────────────────────────────────────────────────────────┐
│                        Backend (Rust/Tauri)                   │
├─────────────────┬─────────────────┬─────────────────┬────────────┤
│   AppConfig      │  Config Commands│  File System    │  App State │
│  (config.json)   │  (RPC Commands) │  (JSON files)   │  (Memory)  │
└─────────────────┴─────────────────┴─────────────────┴────────────┘
```

### Data Flow

**1. App Initialization:**
```
Backend config.json → Tauri RPC → Frontend SettingsStore → Component State
```

**2. Setting Changes:**
```
Component Interaction → SettingsStore → Tauri RPC → Backend config.json
```

**3. Remember Settings Toggle:**
```
rememberSettings: true  → Persist component settings to config.json
rememberSettings: false → Use default values on next app launch
```

### Key Components

**Backend (Rust):**
- `AppConfig` struct in `src-tauri/src/models/config.rs` - Defines all configurable settings
- `config.json` file - Persistent storage for user preferences
- Tauri commands - Handle config loading/saving via RPC

**Frontend (Svelte):**
- `SettingsStore` - Manages persistent settings with reactive state
- `SessionState` - Handles runtime/app state (not persisted)
- `appSetup.ts` - Synchronizes state between frontend and backend
- Components - Use settings and trigger updates

**Settings Categories:**

| Category | Examples | Persistence | Storage Location |
|----------|----------|-------------|------------------|
| **Critical** | Theme, Locale, Notes Folder | Always | `config.json` |
| **Component** | Search Mode, Sidebar State, Thread Mode | Conditional | `config.json` (when enabled) |
| **Runtime** | Active Note, Popup State | Never | Memory only |

### Remember Settings Mechanism

The "Remember Component Settings" toggle controls whether component-level preferences are persisted:

**When ENABLED (`rememberSettings: true`):**
- Component settings are saved to `config.json`
- Settings persist across app restarts
- User preferences are maintained

**When DISABLED (`rememberSettings: false`):**
- Component settings use default values on app launch
- Current session settings remain active until restart
- Provides a "clean slate" experience for each session

### State Synchronization Process

1. **App Launch:**
   - Backend loads `config.json`
   - Frontend receives full state via Tauri RPC
   - `syncSettingsState()` applies values conditionally:
     - Critical settings: Always applied
     - Component settings: Applied only if `rememberSettings: true`

2. **Setting Update:**
   - Component calls `settings.save({ ... })`
   - Frontend updates `SettingsStore` state
   - Tauri RPC sends changes to backend
   - Backend updates `config.json` (if `rememberSettings: true`)

3. **Toggle Remember Settings:**
   - User changes `rememberSettings` preference
   - Current component settings are NOT immediately reset
   - On next launch, component settings use defaults if disabled

This architecture ensures that:
- Critical settings are always preserved
- Component settings respect user preferences
- The system remains responsive and consistent
- Changes are persisted appropriately based on user choice

## Step-by-Step Guide

### 1. Add the setting to the backend config

**File:** `src-tauri/src/models/config.rs`

Add new setting to the `AppConfig` struct:

```rust
/// New setting description.
pub setting_name: String, // or appropriate type
```

Add it to the default config:

```rust
setting_name: "default_value".to_string(),
```

### 2. Add the setting to the frontend interfaces

**File:** `src/lib/interfaces/settings.ts`

Add to `AppSettings` interface:

```typescript
settingName: string; // or appropriate type
```

**File:** `src/lib/interfaces/appState.ts`

Add to `AppPayload` interface:

```typescript
settingName: string; // or appropriate type
```

### 3. Add the setting to the response types

**File:** `src-tauri/src/models/response_types.rs`

Add to `ConfigResponse` struct:

```rust
pub setting_name: String,
```

Add to `AppPayload` struct:

```rust
pub setting_name: String,
```

### 4. Add the setting to the SettingsStore

**File:** `src/lib/stores/settings.svelte.ts`

Add the state property:

```typescript
settingName = $state<string>("default_value");
```

Add to the `serialize()` method:

```typescript
settingName: this.settingName,
```

### 5. Update the app setup synchronization

**File:** `src/lib/utils/appSetup.ts`

Add to `syncSettingsState()` function:

```typescript
// If rememberSettings is true, use saved value; otherwise use default
settings.settingName = state.rememberSettings
    ? state.settingName
    : "default_value";
```

### 6. Update the backend commands

**File:** `src-tauri/src/commands/settings.rs`

Add to `update_config` command:

```rust
config.setting_name = new_config.setting_name;
```

Add to `reset_config_to_defaults` command (but preserve if rememberSettings is off):

```rust
// Don't reset this if user turned off rememberSettings
// config.setting_name = default_config.setting_name;
```

### 7. Add the setting to setup commands

**File:** `src-tauri/src/commands/setup.rs`

Add to the config cloning:

```rust
setting_name: config.setting_name.clone(),
```

Add to the AppPayload creation:

```rust
setting_name: config.setting_name,
```

### 8. Create a method to save the setting (optional)

**File:** `src/lib/stores/settings.svelte.ts`

Add a convenience method:

```typescript
async saveYourSettingName(value: string): Promise<boolean> {
    if (this.rememberSettings) return await this.save({ settingName: value });
    this.settingName = value;
    return true;
}
```

### 9. Use the setting in components

In the component, use the setting and save it when changed:

```svelte
<script lang="ts">
import { settings } from '$lib/stores/settings.svelte';

const handleChange = async (newValue) => {
    settings.settingName = newValue;
    await settings.saveYourSettingName(newValue);
    // or: await settings.save({ yourSettingName: newValue });
};
</script>
```

## Files Modified for Thread Shortcuts Mode Example

For reference, here are all the files that were modified to add the thread shortcuts mode setting:

### Backend Files:
- `src-tauri/src/models/config.rs` - Added config field and default
- `src-tauri/src/models/response_types.rs` - Added to response structs
- `src-tauri/src/commands/settings.rs` - Updated config commands
- `src-tauri/src/commands/setup.rs` - Updated setup commands

### Frontend Files:
- `src/lib/interfaces/settings.ts` - Added to AppSettings interface
- `src/lib/interfaces/appState.ts` - Added to AppPayload interface
- `src/lib/stores/settings.svelte.ts` - Added state and methods
- `src/lib/utils/appSetup.ts` - Added synchronization logic
- `src/lib/components/ThreadShortcutsModeToggle.svelte` - Updated component to use settings

This pattern ensures that new settings integrate seamlessly with TodayNote's settings system while respecting the user's preference for remembering component settings.