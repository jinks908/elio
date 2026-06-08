# Fork of elio-fm/elio

## Changes from Upstream
See [elio-fm/elio](https://github.com/elio-fm/elio)

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
