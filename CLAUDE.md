# CoW MEV Defense — Claude Code Notes

## Black screen after UI changes

**Symptom:** Game renders a completely black canvas after touching tooltip or UI query code.

**Root cause:** Bevy detects mutable aliasing between two queries in the same system at app build-time and panics before anything renders. The panic is silent in the browser — it just produces a black screen.

**Rule:** Any two queries in the same system that both access a component mutably (e.g. `&mut Visibility`, `&mut Transform`, `&mut Text2d`) MUST have filters that statically prove they are disjoint. Bevy cannot infer this from entity data at runtime — it must be provable from the filter types alone.

**The pattern that breaks things:**

```rust
// BAD — both queries access &mut Visibility; Bevy can't prove they won't overlap
mut panel_q:  Query<(&mut Transform, &mut Visibility), With<TooltipPanel>>,
mut lines_q:  Query<(&TooltipLine,   &mut Visibility)>,           // missing Without<TooltipPanel>
```

**The fix — always add cross-`Without` filters:**

```rust
// GOOD — Without<TooltipPanel> on lines_q proves the sets are disjoint
mut panel_q:  Query<(&mut Transform, &mut Visibility), With<TooltipPanel>>,
mut lines_q:  Query<(&TooltipLine,   &mut Visibility), Without<TooltipPanel>>,
```

**When there are multiple panel types** (e.g. `TowerTooltipPanel` + `ShopTooltipPanel`), every query must exclude every other query's marker component it could conflict with:

```rust
// Tower panel: exclude shop panel so &mut Transform/Visibility don't alias
mut tower_panel_q: Query<..., (With<TowerTooltipPanel>, Without<ShopTooltipPanel>)>,
mut shop_panel_q:  Query<..., (With<ShopTooltipPanel>,  Without<TowerTooltipPanel>)>,

// Line queries: exclude BOTH panels AND the other line type
mut tower_lines_q: Query<(&TowerTooltipLine, &mut Visibility, ..),
    (Without<TowerTooltipPanel>, Without<ShopTooltipPanel>, Without<ShopTooltipLine>)>,
mut shop_lines_q:  Query<(&ShopTooltipLine,  &mut Visibility, ..),
    (Without<TowerTooltipPanel>, Without<ShopTooltipPanel>, Without<TowerTooltipLine>)>,
```

**Checklist before adding or editing any system with multiple queries:**

1. List every component accessed mutably (`&mut X`) in each query.
2. For each pair of queries that share a mutable component, verify one has `Without<MarkerOfTheOther>`.
3. `With<A>` alone does NOT prove disjointness from `With<B>` — Bevy's checker assumes any entity could have both unless explicitly excluded.
4. Run `cargo check` after every UI/query change and confirm it builds before rebuilding WASM.

## Project stack

- Bevy 0.18.1, targeting WebAssembly via Trunk / wasm32-unknown-unknown
- WebGL2 renderer — some Bevy rendering features behave differently than native

## Other WASM gotchas

- `Sprite` children that start `Visibility::Hidden` may never appear after being set `Visible`. Always start them `Visibility::Visible` and hide via a system.
- `OnEnter(DefaultState)` fires during Startup — before user `Startup` systems run. Do not rely on resources populated by `Startup` systems inside `OnEnter` handlers for the default state; load assets directly instead.
