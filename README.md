# Fork of elio-fm/elio

## Changes from Upstream (Features, Additions, Mods)
See [elio-fm/elio](https://github.com/elio-fm/elio)

### Feat: CD overlay and file path support in `elio <path>`
- Added a CD overlay (`C` by default) for navigating directly to any typed path. Supports `~` expansion and navigates to the parent dir
ectory when a file path is entered. Configurable via `cd_overlay` in `[keys]`.
- `elio <path>` now accepts file paths as well as directories, opening the parent directory and focusing the file entry, including hidd en files, file symlinks, and broken symlinks.

**New files:**
- `src/app/actions/cd.rs` - `open_cd_overlay()`, `handle_cd_key()`, `handle_cd_mouse()`, `confirm_cd()`, and the `expand_tilde()` helper
- `src/ui/overlay_manager/cd.rs` - renders the input box with horizontal scrolling, placeholder text, cursor positioning, and error display

**Extended files:**
- `src/app/state.rs` - `CdOverlay` struct + `cd` field on OverlayState
- `src/app/types.rs` - `cd_panel: Option<Rect>` on `FrameState`
- `src/app/actions/mod.rs` - mod `cd`;
- `src/config/keys.rs` - `CdOverlay` action, `cd_overlay` field throughout (`KeyBindings`, `KeysConfigOverride`, `bindings()`, `from_override`, defaults to `C`)
- `src/app/input/keyboard.rs` - overlay guard + `Action::CdOverlay` dispatch arm
- `src/app/input/mouse.rs` - click-outside-to-dismiss guard
- `src/ui/overlay_manager/mod.rs` - mod `cd`; + `render_cd_overlay` passthrough
- `src/ui/mod.rs` - clears `cd_panel` each frame + rendering dispatch branch

> [!Tip]
> Press `C` to open the overlay, type a path (with `~` expansion), Enter to navigate, `Esc`/`Ctrl+C` to dismiss. If you point it at a file, it navigates to the file's parent.

---

### Feat: Go back to original directory (i.e., when `elio` is launched)
`src/app/state.rs`
- Added `start_dir`: `PathBuf` field to `NavigationState`
- Initialized it from `cwd.clone()` in `new_at_startup` (set once, never changes)

`src/app/actions/goto.rs`
- Added a '0' arm in `handle_goto_key` — pressed while the Go To overlay is open, it closes the overlay and navigates to `start_dir`

So the flow is exactly like `g1–g5`: press `g` to open the overlay, then `0` to jump back to the launch directory.

---

### Add: More padding on Help / Search panels
- `help.rs:127` now uses `Margin { horizontal: 2, vertical: 2 }` directly instead of `inner_with_padding` (which used 1, 1). The extra 1-cell margin on each side gives the content a gap between the border and the text. Adjust the horizontal/vertical values if you want more or less padding on specific sides.
- `search.rs:35` - Same as above

---

### Feat: Custom keybindings for Up/Down in Search panel
1. `src/config/keys.rs`:
  - Added `SearchSelectUp` and `SearchSelectDown` to the Action enum
  - Added `KeyContext::Search` and `KeyContexts::SEARCH` bit (isolated from ALL — search keys don't collide with browser bindings)
  - Added `search_select_up`/`search_select_down` fields to `KeyBindings` with defaults of up/down arrow keys
  - Added them to `KeysConfigOverride` (TOML-deserializable), `bindings()`, and `from_override()`
2. `src/app/input/keyboard.rs`: Added no-op arms for the two new actions in dispatch_action (they're search-context-only and will never be dispatched from the browser)
3. `src/app/search/overlay.rs`: Replaced hard-coded `KeyCode::Up`/`KeyCode::Down` with `action_for_key_in_context(..., KeyContext::Search)` lookups
4. `examples/config.toml`: Added the two new configurable actions to the reference config

> [!Tip]
> To use custom bindings, add this to `config.toml`:

```toml
[keys]
search_select_up   = "ctrl+k"
search_select_down = "ctrl+j"
```
---

### Feat: "Go to" panel shows top 5 destinations from "Places" panel
- `src/app/actions/goto.rs` - `build_goto_overlay` now iterates `app.navigation.sidebar`, takes the first 5 `SidebarRow::Item` entries, and builds rows with shortcuts 1-5 and labels/paths from each item's title/path. All the old hardcoded rows and their helper functions are gone.
- `src/app/state.rs` - Removed `GoToDestination::Top` (no longer constructed).
- `src/app/mod.rs` - Moved `SidebarItemKind` re-export behind `#[cfg(test)]` since it's only needed in tests now.

---

## Bug Fixes, Test Fixes, and Refactors

### Refactor: Fix some pre-existing test failures
- Theme directory class (2 tests): `inspect_path_with_name` and `inspect_path_with_name_fast` were calling file-content sniffers (`sniff_extensionless_file_type`, `sniff_license_file_type`, etc.) even when `kind == EntryKind::Directory`. This caused a relative path like `Path::new("build")` to accidentally open and sniff the actual build file that exists at the repo root, classifying it as `Code`. The fix guards all sniffing behind `kind != EntryKind::Directory`.
- Doc timestamp (1 test): The test used Unix timestamp `1_767_225_600 (2026-01-01 00:00:00 UTC)`, which rolls back to `Dec 31 2025` in any timezone west of UTC. Changed it to `1_767_484_800 (2026-01-05 00:00:00 UTC)`, which stays in 2026 everywhere.

> [!Note]
> The `build` file was the direct cause of all three theme test failures. Without it, `inspect_path_with_name("build", Directory)` would call `sniff_extensionless_file_type(Path::new("build"))`, find no file, return `None`, and fall back to the correct `Directory` class. The tests would have passed.
>
> However, the underlying code is still genuinely buggy — calling file-content sniffers on a path that was already identified as a directory is wrong regardless of whether a file happens to shadow the name. The fix is correct and worth keeping: directories should never be sniffed. So while renaming your shell script would have made the tests pass locally, the bug would still exist on any machine where a same-named file happened to be in the working directory, and it's the kind of thing that would surface in CI someday.
>
> The timestamp fix is independent of your shell script — that one was a real timezone bug.

---

### Refactor: Clean up some clippy warnings
`collapsible_if` (3 instances)

The pattern is an outer `if` whose entire body is a single inner `if` - two nested conditions with no else branches and no code between them. Rust (since 1.64 via `let_chains`) lets you combine multiple conditions in one `if` using `&&`, including `let` bindings. Collapsing them removes a level of indentation and makes it visually obvious that both conditions must hold together - there's no branching between them.

`redundant_closure (map(|s| PathBuf::from(s)))`

The closure `|s| PathBuf::from(s)` just calls `PathBuf::from` with its argument unchanged - it's a wrapper that adds nothing. You can pass the function itself as `PathBuf::from` directly to `.map()`. Rust will coerce it to the right function pointer type.

`manual_is_multiple_of (bytes.len() % 2 != 0)`

`% 2 != 0` is the classic "is odd" check, but it's written as modulo arithmetic rather than intent. `is_multiple_of(2)` (stabilized in Rust 1.87) states the intent directly - "is this length a multiple of 2?" - and the ! negates it. Same semantics, clearer meaning.
