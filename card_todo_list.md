# Card Effect TODO List

> Status: ✅ implemented | ❌ not yet | ➖ plain damage (no effect needed) | 🔵 simplified no-op

## Ability Effects — 0 remaining

All ability effect_ids have dispatch handlers. Cards using `ability_placeholder` are Pokemon whose GDScript counterpart has no registered effect_id — they use vanilla abilities or the JSON data doesn't track effect_ids.

| Status | Decks | Card | Ability | Description |
|--------|-------|------|---------|-------------|
| ✅ | 17 | Fezandipiti ex | Flip the Script | Draw 3 on KO |
| ✅ | 11 | Radiant Greninja | Concealed Cards | Discard energy, draw 2 |
| ✅ | 9 | Squawkabilly ex | Squawk and Seize | Discard hand, draw 6 |
| ✅ | 9 | Lumineon V | Luminous Sign | Search deck for Supporter |
| ✅ | 9 | Rotom V | Instant Charge | Draw 3, end turn |
| ✅ | 8 | Mew ex | Restart | Redraw hand |
| ✅ | 6 | Manaphy | Awaken | Bench protection |
| ✅ | 5 | Iron Bundle | Blower | Gust opponent from bench |
| ✅ | 4 | Dusknoir | Curse Bomb | Self-KO, 13 counters on opponent |
| ✅ | 4 | Dusclops | Curse Bomb | Self-KO, 5 counters on opponent |
| ✅ | 3 | Miraidon ex | Tandem Unit | Search 2 Lightning basics |
| ✅ | 2 | Charizard ex | Infernal Reign | Attach 3 Fire from deck |
| ✅ | 2 | Pidgeot ex | Wind Search | Search any 1 card |
| ✅ | 2 | Gardevoir ex | Psychic Embrace | Attach Psychic from discard |
| 🔵 | 6 | Munkidori | Excited Brain | Move up to 3 damage counters |
| 🔵 | 6 | Bloodmoon Ursaluna ex | Veteran's Technique | Attack cost reduced by opponent's prizes taken |
| 🔵 | 4 | Drakloak | Recon Directive | Look top 2, take 1 |
| 🔵 | 4 | Entei V / Raikou V | Fleet Foot | Draw 1 if active |
| 🔵 | 4 | Radiant Alakazam | Painful Spoons | Move 2 damage counters |
| 🔵 | 3 | Radiant Gardevoir | Loving Veil | HP/weakness modifier |
| 🔵 | 3 | Kirlia | Refinement | Draw/discard (deck filter) |
| 🔵 | 2 | Origin Dialga VSTAR | Star Chronos | Extra turn (VSTAR) |
| 🔵 | 2 | Arceus V | Charging Star | Attach 3 energy from deck |
| 🔵 | 2 | Banette ex | Cursed Domain | Item lock ability |
| 🔵 | 1 | Many others... | _(various)_ | Low-frequency/niche cards |

## Attack Effects

### With effect dispatches ✅

| Decks | Card | Attack | Effect |
|-------|------|--------|--------|
| 3 | Miraidon ex | Photon Blaster (220) | Self-lock next turn |
| 3 | Iron Hands ex | Double Impact (120+30) | Bench snipe 30 |
| 2 | Charizard ex | Scorching Darkness (180+) | +30 per opponent prize |
| 2 | Pidgeot ex | Gale Winds (120) | Optional discard stadium → bench 120 |
| 2 | Radiant Greninja | Moonlight Shuriken (90) | Hit active + bench for 90 each |
| 1 | Radiant Charizard | Combustion Blast (250) | Only usable with ≤1 prize remaining |
| 2 | Gouging Fire ex | Magma Blast (220) | Discard 1 energy from self |
| 2 | Roaring Moon ex | Calamity Storm (200+) | +60 if stadium in play |
| 4 | Dragapult ex | Phantom Dive (200) | +6 damage counters on bench |

### High-priority missing attack effects ❌

Attacks on key Pokemon that appear in 2+ decks, have damage >0, and have real PTCG effects:

| Priority | Decks | Card | Attack | Damage | Effect Needed |
|----------|-------|------|--------|--------|---------------|
| 🔴 | 7 | Dragapult ex | Phantom Dive | 200 | Place 6 bench counters (dispatch exists, check effect_id wiring) |
| 🔴 | 3 | Charizard ex (CSV5C) | Burning Darkness | 180+ | Bonus per opponent prize taken |
| 🔴 | 3 | Giratina VSTAR | Lost Impact | 280 | Discard 2 energy from self, KO check |
| 🔴 | 2 | Arceus VSTAR | Trinity Nova | 200 | Search 2 energy, attach to bench |
| 🔴 | 2 | Lugia VSTAR | Storm Dive | 220 | Search Archeops from deck |
| 🔴 | 2 | Origin Palkia VSTAR | Subspace Swell | 60+ | +20 per bench Pokemon (both players) |
| 🔴 | 2 | Regidrago VSTAR | Dragon's Glory | — | Copy any Dragon Pokemon's attack from discard |
| 🟡 | 2 | Gholdengo ex | Coin Scramble | 50× | 50 per energy discarded from hand |
| 🟡 | 2 | Raging Bolt ex | Bellowing Thunder | 0 | Discard energy for damage |
| 🟡 | 2 | Banette ex | Eternal Darkness | 30 | Item lock next turn |
| 🟡 | 2 | Gardevoir ex | Miracle Force | 160 | Heal 30 from this Pokemon |
| 🟡 | 1 | Iron Thorns ex | Volt Cyclone | 140 | Discard Future Energy for bonus |
| 🟡 | 1 | Chien-Pao ex | Hail Blade | 60× | 60 per Water energy discarded |
| 🟡 | 1 | Blissey ex | Return | 180 | Heal 30 from bench |
| 🟡 | 1 | Hisuian Samurott VSTAR | Cruel Blade | 110+ | +110 if defender damaged |

### Low-priority / plain damage ➖

All other attacks (82 total listed as ❌ in the raw scan) are plain damage attacks with no PTCG effect text. They work correctly with the existing plain-damage fallback. No implementation needed.

## Trainer Effects — Status Summary

| Status | Decks | Card | Effect |
|--------|-------|------|--------|
| ✅ | 13+ | Nest Ball | Search basic to bench |
| ✅ | 12+ | Ultra Ball | Discard 2, search any Pokemon |
| ✅ | 10+ | Arven | Search Item + Tool |
| ✅ | 10+ | Boss's Orders | Gust opponent bench |
| ✅ | 10+ | Iono | Both shuffle hand, draw by prizes |
| ✅ | 6+ | Electric Generator | Top 5, attach Lightning |
| ✅ | 6+ | Buddy-Buddy Poffin | Search 2 basics HP≤70 to bench |
| ✅ | 6+ | Super Rod | Shuffle 3 Pokemon+Energy from discard |
| ✅ | 6+ | Counter Catcher | Gust opponent bench (same as Boss) |
| ✅ | 6+ | Rare Candy | Evolve Stage1→Stage2 directly |
| 🔵 | 8+ | Lost Vacuum | Discard Tool or Stadium |
| 🔵 | 5+ | Earthen Vessel | Discard 1, search 2 basic energy |
| 🔵 | 5+ | Night Stretcher | Retrieve Pokemon/Energy from discard |
| 🔵 | 5+ | Hisuian Heavy Ball | Search prize cards |
| 🔵 | 4+ | Ciphermaniac's Codebreaking | Shuffle hand, draw |
| 🔵 | 4+ | Secret Box | Discard 2, search Item+Tool+Supporter |
| 🔵 | 4+ | Unfair Stamp | Opponent shuffles, draws 2 |
| 🔵 | 4+ | Switch Cart | Switch basic + heal 30 |
| 🔵 | 4+ | Cyllene | Flip 2 coins, recover from discard |
| 🔵 | 4+ | Professor Turo's Scenario | Return active to hand |
| 🔵 | 3+ | Thorton | Replace bench from discard |
| 🔵 | 3+ | Prof Sada's Vitality | Attach energy to Ancient Pokemon |
| 🔵 | 3+ | Dark Patch | Attach Darkness from discard |
| 🔵 | 3+ | Energy Switch | Move energy between Pokemon |
| 🔵 | 3+ | Pal Pad | Shuffle 2 Supporters from discard |
| 🔵 | 3+ | Professor's Research | Discard hand, draw 7 |
| 🔵 | 2+ | Tech Radar | Discard 1, search 2 Future Pokemon |
| 🔵 | 2+ | Pokegear 3.0 | Look top 7, reveal Supporter |
| 🔵 | 2+ | TM Evolution | Search deck for evolution |
| 🔵 | 2+ | TM Devolution | Devolve opponent's Pokemon |
| 🔵 | 1+ | Energy Sticker | Flip coin, attach energy from discard |

## Summary

| Category | Implemented | Simplified | Total |
|----------|------------|------------|-------|
| Ability effects | 16 | ~30 (niche/vanilla) | ~46 |
| Attack effects | 9 | 82 (plain damage) | ~90 |
| Trainer effects | 7 | ~25 | ~32 |
| **Total** | **32** | **~137** | **~169** |

### Next Priority

1. **🔴 P0** (this session): Dragapult ex effect_id wiring fix
2. **🔴 P1**: Charizard Burning Darkness, Arceus VSTAR Trinity Nova (core archetypes)
3. **🔴 P2**: Lugia VSTAR, Regidrago VSTAR (defining attacks for major archetypes)
4. **🟡 P3**: Gardevoir, Banette, Blissey effects
5. **🟡 P4**: Key trainer effects (Lost Vacuum, Switch Cart, TM Evolution)
6. **➖ P5**: Remaining 82 plain-damage attacks (no work needed)
