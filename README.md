# Fork of elio-fm/elio

## Changes from Upstream
See [elio-fm/elio](https://github.com/elio-fm/elio)

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

### Refactor: Clean up some clippy warnings
`collapsible_if` (3 instances)

The pattern is an outer `if` whose entire body is a single inner `if` - two nested conditions with no else branches and no code between them. Rust (since 1.64 via `let_chains`) lets you combine multiple conditions in one `if` using `&&`, including `let` bindings. Collapsing them removes a level of indentation and makes it visually obvious that both conditions must hold together - there's no branching between them.

`redundant_closure (map(|s| PathBuf::from(s)))`

The closure `|s| PathBuf::from(s)` just calls `PathBuf::from` with its argument unchanged - it's a wrapper that adds nothing. You can pass the function itself as `PathBuf::from` directly to `.map()`. Rust will coerce it to the right function pointer type.

`manual_is_multiple_of (bytes.len() % 2 != 0)`

`% 2 != 0` is the classic "is odd" check, but it's written as modulo arithmetic rather than intent. `is_multiple_of(2)` (stabilized in Rust 1.87) states the intent directly - "is this length a multiple of 2?" - and the ! negates it. Same semantics, clearer meaning.

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
