# Unimplemented Features

Features from the latest PtcgDeckAgent (`8a66870`, 2026-05-05) not yet in the Rust engine. Categorized by priority for the RL training goal (Miraidon ex vs Charizard ex / Pidgeot ex).

## Should Implement (affects core matchup correctness)

### Engine / Rules

- [x] ~~Between-turns pokemon_check (poison/burn/sleep/paralyze)~~ — `engine.rs:do_pokemon_check()`
- [x] ~~Heavy Baton KO energy transfer~~ — `engine.rs:process_ko()`
- [x] ~~Forest Seal Stone — Star Alchemy (tool-granted ability)~~ — `card.rs` + `engine.rs`
- [x] ~~Deck-out loss condition~~ — `engine.rs:check_winner()`
- [x] ~~First-turn attack restriction~~ — `rules.rs:validate_attack()`

### Pokemon Abilities (in deck, dispatch missing)
- [ ] ability_concealed_cards (Radiant Greninja) — discard 1 energy, draw 2
- [ ] ability_restart (Mew ex) — hand → deck bottom, draw same count

### Pokemon Attacks (in deck, effect not implemented)
- [ ] AttackMoonlightShuriken (Radiant Greninja) — 90 dmg to up to 2 opponent Pokemon
- [ ] AttackGaleWinds (Pidgeot ex) — 120 dmg + optional discard stadium for bench 120

### Trainer Effects (in deck, dispatch missing)
- [ ] Buddy-Buddy Poffin — search up to 2 HP ≤ 70 basics to bench
- [ ] Super Rod — shuffle up to 3 Pokemon + basic energy from discard pile to deck
- [ ] Counter Catcher — switch opponent bench Pokemon to active
- [ ] Switch Cart — switch active basic to bench, heal 30 (if basic)
- [ ] Lost Vacuum — discard a tool or stadium from play

### Trainer Effects (in deck, simplified no-op)
- [ ] Ciphermaniac's Codebreaking — shuffle hand, draw (opponent also redraws)
- [ ] Earthen Vessel — discard 1, search 2 basic energy from deck
- [ ] Hisuian Heavy Ball — search from prize cards (complex lookup)
- [ ] Night Stretcher — retrieve Pokemon or basic energy from discard
- [ ] Secret Box — discard 2, search item + tool + supporter
- [ ] Unfair Stamp — opponent shuffles hand, draws 2 (when behind on prizes)
- [ ] Cyllene — flip 2 coins, for each heads, recover from discard
- [ ] Professor Turo's Scenario — return active to hand (all cards)
- [ ] Thorton — shuffle bench Pokemon into deck, replace from discard

## Skip (not needed for current RL training goal)

### New cards (added in PtcgDeckAgent since baseline c3b0161)
| Card | Reason to skip |
|------|---------------|
| Magneton CBB5C_0301 | Miraidon variant card, not in core matchup |
| Magnemite CSV1C_042 | Magneton pre-evolution |
| Magnemite SVP_102 | Magneton pre-evolution (alt art) |
| Gapejaw Bog CS5.5C_066 | Stadium, not in current deck templates |
| Hisuian Samurott VSTAR CS5aC_086 | Different archetype (Darkness) |
| Hisuian Samurott V CSNC_007 | Different archetype (Darkness) |
| Energy Sticker CSV5C_114 | Item, not in current deck templates |
| Cycling Road CSV6C_127 | Stadium, not in current deck templates |
| Roaring Moon CSV7C_143 | Requires Ancient tag system |
| Chansey CSV8C_164 | Different archetype (Blissey tank) |

### New effects (added since c3b0161)
| Effect | Reason to skip |
|--------|---------------|
| AbilityOvervoltDischarge | Magneton only (not in core matchup) |
| AbilityPlaceDamageCountersVSTAR | Hisuian Samurott VSTAR only |
| AttackAncientDiscardCountDamage | Roaring Moon only (needs Ancient tag) |
| AttackBonusIfDefenderDamaged | Hisuian Samurott VSTAR only |
| AttackAttachBasicEnergyFromHandToOwnPokemon | Chansey only |
| EffectCyclingRoad | Stadium, not in core matchup |
| EffectGapejawBog | Stadium, not in core matchup |
| EffectEnergySticker | Item, not in core matchup |

### Trainer effects (not in current deck templates)
| Effect | Reason to skip |
|--------|---------------|
| Accompanying Flute | Not in current decks |
| Cancel Cologne | Not in current decks |
| Capturing Aroma | Not in current decks |
| Carmine | Not in current decks |
| Cheren's Care | Not in current decks |
| Colress's Experiment | Not in current decks |
| Colress's Tenacity | Not in current decks |
| Crushing Hammer | Not in current decks |
| Dark Patch | Not in current decks |
| Energy Switch | Not in current decks |
| Enhanced Hammer | Not in current decks |
| Eri | Not in current decks |
| Erika's Invitation | Not in current decks |
| Giacomo | Not in current decks |
| Hyper Aroma | Not in current decks |
| Irida | Not in current decks |
| Jacq | Not in current decks |
| Janine's Secret Art | Not in current decks |
| Kieran | Not in current decks |
| Lance | Not in current decks |
| Letter of Encouragement | Not in current decks |
| Mela | Not in current decks |
| Mirage Gate | Not in current decks |
| Miss Fortune Sisters | Not in current decks |
| Pal Pad | Not in current decks |
| Penny | Not in current decks |
| Pokemon Catcher | Not in current decks (use Counter Catcher) |
| Prime Catcher | Not in current decks |
| Roseanne's Backup | Not in current decks |
| Roxanne | Not in current decks |
| Sada's Vitality | Not in current decks |
| Salvatore | Not in current decks |
| Serena | Not in current decks |
| Team Yell's Cheer | Not in current decks |
| Techno Radar | Not in current decks |
| Trekking Shoes | Not in current decks |
| Xerosic's Machinations | Not in current decks |
| TM Devolution | Not in current decks |
| TM Turbo Energize | Not in current decks |
| TM Crisis Punch | Not in current decks |
| Energy Sticker | New card, not in current decks |

### Stadium effects (not in current deck templates)
| Effect | Reason to skip |
|--------|---------------|
| Artazon | Not in current decks |
| Collapsed Stadium | In Charizard deck but effect is minor (bench limit) |
| Cycling Road | New card, not in current decks |
| Gapejaw Bog | New card, not in current decks |
| Gravity Mountain | In Miraidon deck but effect is minor |
| Jamming Tower | Not in current decks |
| League HQ | Not in current decks |
| Lost City | Not in current decks |
| Magma Basin | Not in current decks |
| Mesagoza | Not in current decks |
| Moonlit Hill | Not in current decks |
| Temple of Sinnoh | Not in current decks |
| Town Store | Not in current decks |

### Tool effects (not in current deck templates, except where noted)
| Effect | Reason to skip |
|--------|---------------|
| Binding Mochi | Not in current decks |
| Emergency Jelly | Not in current decks |
| Exp. Share | Not in current decks |
| Handheld Fan | Not in current decks |
| Luxurious Cape | Not in current decks |
| Sparkling Crystal | Not in current decks |
| Future Boost | Not in current decks |

### System-level features

| Feature | Reason to skip |
|---------|---------------|
| Ancient tag system | Only Roaring Moon / Sada's Vitality |
| Tera trait | Only Sparkling Crystal interaction |
| VSTAR power once-per-game tracking | Only Forest Seal Stone (can use ability tracking) |
| Confusion coin flip at attack | Status system partially implemented |
| Lost Zone mechanics (Lost City, etc.) | Not in current matchups |
| LLM deck strategies (7 strategies) | AI layer, not engine |
| Deck coach recommendations | UI feature |
| Replay browser / Tournament mode | Game app features |
| Community page / Xiaohongshu integration | App marketing features |
| Android APK export / macOS window size | Platform features |
| Card feedback rules | Beta testing feature |

### Deck strategies (GDScript AI — not engine layer)

22 registered deck strategies in PtcgDeckAgent. The Rust engine only implements 2:
- Miraidon ex (core matchup)
- Charizard ex / Pidgeot ex (core matchup)

Other 20+ strategies exist as AI guidance in GDScript but are not needed for the Rust engine unless new matchups are added.

## Summary

| Priority | Count | Status |
|----------|-------|--------|
| Engine/Rules fixes | 5 | ✅ All done |
| Core Pokemon effects | 4 | 2 done (Star Alchemy, Infernal Reign), 2 pending (Concealed Cards, Restart) |
| Core Attack effects | 2 | 2 pending (Moonlight Shuriken, Gale Winds) |
| Core Trainer effects | 9 | 5 pending (Buddy Poffin, Super Rod, Counter Catcher, Switch Cart, Lost Vacuum) |
| Edge trainer effects | 9 | Skip (not in core decks) |
| New cards (10) | 10 | Skip |
| New effects (8) | 8 | Skip |
| Other trainer effects (~40) | 40 | Skip |
| Stadium effects (13) | 13 | Skip |
| System features | 8 | Skip |
