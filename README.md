# Fork of elio-fm/elio

## Changes from Upstream
See [elio-fm/elio](https://github.com/elio-fm/elio)

### Feat: "Go to" panel shows top 5 destinations from "Places" panel
- `src/app/actions/goto.rs` - `build_goto_overlay` now iterates `app.navigation.sidebar`, takes the first 5 `SidebarRow::Item` entries, and builds rows with shortcuts 1-5 and labels/paths from each item's title/path. All the old hardcoded rows and their helper functions are gone.
- `src/app/state.rs` - Removed `GoToDestination::Top` (no longer constructed).
- `src/app/mod.rs` - Moved `SidebarItemKind` re-export behind `#[cfg(test)]` since it's only needed in tests now.
