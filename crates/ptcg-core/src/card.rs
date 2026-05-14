//! Card definitions and static data

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;

/// Unique identifier for a card definition (set_code + card_index)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardDefId(pub String);

impl CardDefId {
    pub fn new(set_code: &str, index: &str) -> Self {
        Self(format!("{}_{}", set_code, index))
    }
}

impl fmt::Display for CardDefId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Card type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Pokemon,
    Item,
    Supporter,
    Stadium,
    Tool,
    BasicEnergy,
    SpecialEnergy,
}

impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CardType::Pokemon => write!(f, "Pokemon"),
            CardType::Item => write!(f, "Item"),
            CardType::Supporter => write!(f, "Supporter"),
            CardType::Stadium => write!(f, "Stadium"),
            CardType::Tool => write!(f, "Tool"),
            CardType::BasicEnergy => write!(f, "Basic Energy"),
            CardType::SpecialEnergy => write!(f, "Special Energy"),
        }
    }
}

/// Energy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnergyType {
    Fire,
    Water,
    Grass,
    Lightning,
    Psychic,
    Fighting,
    Darkness,
    Metal,
    Dragon,
    Colorless,
}

impl EnergyType {
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'R' => Some(EnergyType::Fire),
            'W' => Some(EnergyType::Water),
            'G' => Some(EnergyType::Grass),
            'L' => Some(EnergyType::Lightning),
            'P' => Some(EnergyType::Psychic),
            'F' => Some(EnergyType::Fighting),
            'D' => Some(EnergyType::Darkness),
            'M' => Some(EnergyType::Metal),
            'N' => Some(EnergyType::Dragon),
            'C' => Some(EnergyType::Colorless),
            _ => None,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            EnergyType::Fire => 'R',
            EnergyType::Water => 'W',
            EnergyType::Grass => 'G',
            EnergyType::Lightning => 'L',
            EnergyType::Psychic => 'P',
            EnergyType::Fighting => 'F',
            EnergyType::Darkness => 'D',
            EnergyType::Metal => 'M',
            EnergyType::Dragon => 'N',
            EnergyType::Colorless => 'C',
        }
    }
}

/// Pokemon stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Basic,
    Stage1,
    Stage2,
}

/// Pokemon mechanic (ex, V, VSTAR, VMAX, Radiant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mechanic {
    None,
    Ex,
    V,
    Vstar,
    Vmax,
    Radiant,
}

/// Weakness or resistance type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMultiplier {
    pub energy_type: EnergyType,
    pub multiplier: i32, // e.g., 2 for weakness, -30 for resistance
}

/// Attack definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub name: String,
    pub cost: Vec<EnergyType>,
    pub damage: u16,
    pub text: String,
    pub effect_id: Option<String>,
}

/// Ability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub name: String,
    pub text: String,
    pub effect_id: String,
}

/// Static card definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDef {
    pub id: CardDefId,
    pub name: String,
    pub name_en: String,
    pub set_code: String,
    pub card_type: CardType,
    
    // Pokemon-specific fields
    pub stage: Option<Stage>,
    pub hp: Option<u16>,
    pub energy_type: Option<EnergyType>,
    pub weakness: Option<TypeMultiplier>,
    pub resistance: Option<TypeMultiplier>,
    pub retreat_cost: Option<u8>,
    pub mechanic: Option<Mechanic>,
    pub attacks: Vec<Attack>,
    pub abilities: Vec<Ability>,
    
    // Tool/Energy properties
    pub provides_energy: Option<Vec<EnergyType>>,
    pub damage_modifier: Option<i32>,
    pub retreat_modifier: Option<i8>,
}

impl CardDef {
    pub fn is_pokemon(&self) -> bool {
        self.card_type == CardType::Pokemon
    }

    pub fn is_basic_energy(&self) -> bool {
        self.card_type == CardType::BasicEnergy
    }

    pub fn is_special_energy(&self) -> bool {
        self.card_type == CardType::SpecialEnergy
    }

    pub fn is_trainer(&self) -> bool {
        matches!(
            self.card_type,
            CardType::Item | CardType::Supporter | CardType::Stadium | CardType::Tool
        )
    }

    pub fn is_ex(&self) -> bool {
        self.mechanic == Some(Mechanic::Ex)
    }

    pub fn is_v(&self) -> bool {
        self.mechanic == Some(Mechanic::V)
    }

    pub fn is_vstar(&self) -> bool {
        self.mechanic == Some(Mechanic::Vstar)
    }

    pub fn can_be_searched_by_nest_ball(&self) -> bool {
        self.is_pokemon() && self.stage == Some(Stage::Basic)
    }

    pub fn can_be_evolved_from(&self, other: &CardDef) -> bool {
        if !self.is_pokemon() || !other.is_pokemon() {
            return false;
        }
        
        let (self_stage, other_stage) = match (self.stage, other.stage) {
            (Some(s), Some(o)) => (s, o),
            _ => return false,
        };
        
        match (self_stage, other_stage) {
            (Stage::Stage1, Stage::Basic) => true,
            (Stage::Stage2, Stage::Stage1) => true,
            _ => false,
        }
    }
}

/// Card definitions registry
#[derive(Debug, Clone, Default)]
pub struct CardRegistry {
    cards: HashMap<CardDefId, CardDef>,
}

impl CardRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, card: CardDef) {
        let id = card.id.clone();
        self.cards.insert(id, card);
    }

    pub fn get(&self, id: &CardDefId) -> Option<&CardDef> {
        self.cards.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &CardDef> {
        self.cards.values()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

/// Pre-loaded card definitions for supported matchups
pub mod presets {
    use super::*;

    pub fn load_miraidon_charizard_cards() -> CardRegistry {
        let mut registry = CardRegistry::new();
        
        // Basic Energy
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "LIG"),
            name: "Lightning Energy".into(),
            name_en: "Lightning Energy".into(),
            set_code: "CSVE1C".into(),
            card_type: CardType::BasicEnergy,
            stage: None,
            hp: None,
            energy_type: Some(EnergyType::Lightning),
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: Some(vec![EnergyType::Lightning]),
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "FIR"),
            name: "Fire Energy".into(),
            name_en: "Fire Energy".into(),
            set_code: "CSVE1C".into(),
            card_type: CardType::BasicEnergy,
            stage: None,
            hp: None,
            energy_type: Some(EnergyType::Fire),
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: Some(vec![EnergyType::Fire]),
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "PSY"),
            name: "Psychic Energy".into(),
            name_en: "Psychic Energy".into(),
            set_code: "CSVE1C".into(),
            card_type: CardType::BasicEnergy,
            stage: None,
            hp: None,
            energy_type: Some(EnergyType::Psychic),
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: Some(vec![EnergyType::Psychic]),
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "DAR"),
            name: "Darkness Energy".into(),
            name_en: "Darkness Energy".into(),
            set_code: "CSVE1C".into(),
            card_type: CardType::BasicEnergy,
            stage: None,
            hp: None,
            energy_type: Some(EnergyType::Darkness),
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: Some(vec![EnergyType::Darkness]),
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "GRA"),
            name: "Grass Energy".into(),
            name_en: "Grass Energy".into(),
            set_code: "CSVE1C".into(),
            card_type: CardType::BasicEnergy,
            stage: None,
            hp: None,
            energy_type: Some(EnergyType::Grass),
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: Some(vec![EnergyType::Grass]),
            damage_modifier: None,
            retreat_modifier: None,
        });

        // Double Turbo Energy (Special Energy)
        registry.register(CardDef {
            id: CardDefId::new("CSNC", "024"),
            name: "Double Turbo Energy".into(),
            name_en: "Double Turbo Energy".into(),
            set_code: "CSNC".into(),
            card_type: CardType::SpecialEnergy,
            stage: None,
            hp: None,
            energy_type: Some(EnergyType::Colorless),
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: Some(vec![EnergyType::Colorless, EnergyType::Colorless]),
            damage_modifier: Some(-20),
            retreat_modifier: None,
        });

        // Basic Pokemon - Miraidon Line
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "050"),
            name: "Miraidon ex".into(),
            name_en: "Miraidon ex".into(),
            set_code: "CSV1C".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Basic),
            hp: Some(220),
            energy_type: Some(EnergyType::Lightning),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1),
            mechanic: Some(Mechanic::Ex),
            attacks: vec![
                Attack {
                    name: "Photon Blaster".into(),
                    cost: vec![EnergyType::Lightning, EnergyType::Lightning, EnergyType::Colorless],
                    damage: 220,
                    text: "220 damage. During your next turn, you can't use this attack.".into(),
                    effect_id: Some("attack_self_lock_next_turn".into()),
                }
            ],
            abilities: vec![
                Ability {
                    name: "Tandem Unit".into(),
                    text: "Once during your turn, you may search your deck for up to 2 Basic Lightning Pokemon and put them on your Bench.".into(),
                    effect_id: "ability_tandem_unit".into(),
                }
            ],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        // Iron Hands ex
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "051"),
            name: "Iron Hands ex".into(),
            name_en: "Iron Hands ex".into(),
            set_code: "CSV6C".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Basic),
            hp: Some(230),
            energy_type: Some(EnergyType::Lightning),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(2),
            mechanic: Some(Mechanic::Ex),
            attacks: vec![
                Attack {
                    name: "Double Impact".into(),
                    cost: vec![EnergyType::Lightning, EnergyType::Lightning, EnergyType::Colorless, EnergyType::Colorless],
                    damage: 120,
                    text: "120 damage. Does 30 damage to 1 of your opponent's Benched Pokemon.".into(),
                    effect_id: Some("attack_bench_snipe_30".into()),
                }
            ],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        // Manaphy
        registry.register(CardDef {
            id: CardDefId::new("CS5bC", "052"),
            name: "Manaphy".into(),
            name_en: "Manaphy".into(),
            set_code: "CS5bC".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Basic),
            hp: Some(70),
            energy_type: Some(EnergyType::Water),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1),
            mechanic: None,
            attacks: vec![],
            abilities: vec![
                Ability {
                    name: "Awaken".into(),
                    text: "Prevent all damage done to your Benched Pokemon by attacks.".into(),
                    effect_id: "ability_awaken".into(),
                }
            ],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        // Basic Pokemon - Charizard Line
        registry.register(CardDef {
            id: CardDefId::new("151C", "004"),
            name: "Charmander".into(),
            name_en: "Charmander".into(),
            set_code: "151C".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Basic),
            hp: Some(70),
            energy_type: Some(EnergyType::Fire),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Water, multiplier: 2 }),
            resistance: None,
            retreat_cost: Some(1),
            mechanic: None,
            attacks: vec![
                Attack {
                    name: "Ember".into(),
                    cost: vec![EnergyType::Fire, EnergyType::Colorless],
                    damage: 50,
                    text: "50 damage. Discard 1 Energy from this Pokemon.".into(),
                    effect_id: None,
                }
            ],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "015"),
            name: "Charmeleon".into(),
            name_en: "Charmeleon".into(),
            set_code: "CSV5C".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Stage1),
            hp: Some(110),
            energy_type: Some(EnergyType::Fire),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Water, multiplier: 2 }),
            resistance: None,
            retreat_cost: Some(2),
            mechanic: None,
            attacks: vec![
                Attack {
                    name: "Raging Claw".into(),
                    cost: vec![EnergyType::Fire, EnergyType::Fire, EnergyType::Colorless],
                    damage: 80,
                    text: "80+ damage. If your opponent's Active Pokemon is a Pokemon V, this attack does 80 more damage.".into(),
                    effect_id: None,
                }
            ],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "075"),
            name: "Charizard ex".into(),
            name_en: "Charizard ex".into(),
            set_code: "CSV5C".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Stage2),
            hp: Some(330),
            energy_type: Some(EnergyType::Fire),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Water, multiplier: 2 }),
            resistance: None,
            retreat_cost: Some(2),
            mechanic: Some(Mechanic::Ex),
            attacks: vec![
                Attack {
                    name: "Scorching Darkness".into(),
                    cost: vec![EnergyType::Fire, EnergyType::Fire, EnergyType::Colorless, EnergyType::Colorless],
                    damage: 180,
                    text: "180+ damage. If your opponent has 3 or fewer Prize Cards remaining, this attack does 180 more damage (for each of their remaining Prize Cards, add 30 more damage).".into(),
                    effect_id: Some("attack_prize_count_damage".into()),
                }
            ],
            abilities: vec![
                Ability {
                    name: "Infernal Reign".into(),
                    text: "When you evolve this Pokemon during your turn, you may search your deck for up to 3 Fire Energy cards and attach them to your Pokemon in any way you like.".into(),
                    effect_id: "ability_infernal_reign".into(),
                }
            ],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        // Pidgeot Line
        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "099"),
            name: "Pidgey".into(),
            name_en: "Pidgey".into(),
            set_code: "CSV4C".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Basic),
            hp: Some(60),
            energy_type: Some(EnergyType::Colorless),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(0),
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "101"),
            name: "Pidgeot ex".into(),
            name_en: "Pidgeot ex".into(),
            set_code: "CSV4C".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Stage2),
            hp: Some(280),
            energy_type: Some(EnergyType::Colorless),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1),
            mechanic: Some(Mechanic::Ex),
            attacks: vec![
                Attack {
                    name: "Gale Winds".into(),
                    cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless],
                    damage: 120,
                    text: "120 damage. You may discard 1 Stadium card from your hand. If you do, this attack does 120 damage to 1 of your opponent's Benched Pokemon.".into(),
                    effect_id: Some("attack_optional_discard_stadium".into()),
                }
            ],
            abilities: vec![
                Ability {
                    name: "Wind Search".into(),
                    text: "Once during your turn, you may search your deck for any 1 card and put it in your hand. Then, shuffle your deck.".into(),
                    effect_id: "ability_wind_search".into(),
                }
            ],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        // Simple Trainer Cards
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "043"),
            name: "Nest Ball".into(),
            name_en: "Nest Ball".into(),
            set_code: "CSV1C".into(),
            card_type: CardType::Item,
            stage: None,
            hp: None,
            energy_type: None,
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "112"),
            name: "Ultra Ball".into(),
            name_en: "Ultra Ball".into(),
            set_code: "CSV1C".into(),
            card_type: CardType::Item,
            stage: None,
            hp: None,
            energy_type: None,
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSVH1C", "045"),
            name: "Rare Candy".into(),
            name_en: "Rare Candy".into(),
            set_code: "CSVH1C".into(),
            card_type: CardType::Item,
            stage: None,
            hp: None,
            energy_type: None,
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "123"),
            name: "Arven".into(),
            name_en: "Arven".into(),
            set_code: "CSV1C".into(),
            card_type: CardType::Supporter,
            stage: None,
            hp: None,
            energy_type: None,
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "123"),
            name: "Iono".into(),
            name_en: "Iono".into(),
            set_code: "CSV3C".into(),
            card_type: CardType::Supporter,
            stage: None,
            hp: None,
            energy_type: None,
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSVH1aC", "023"),
            name: "Boss's Orders".into(),
            name_en: "Boss's Orders".into(),
            set_code: "CSVH1aC".into(),
            card_type: CardType::Supporter,
            stage: None,
            hp: None,
            energy_type: None,
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "107"),
            name: "Electric Generator".into(),
            name_en: "Electric Generator".into(),
            set_code: "CSV1C".into(),
            card_type: CardType::Item,
            stage: None,
            hp: None,
            energy_type: None,
            weakness: None,
            resistance: None,
            retreat_cost: None,
            mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None,
            damage_modifier: None,
            retreat_modifier: None,
        });

        // --- Pokemon: Miraidon deck allies ---
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "020"),
            name: "Radiant Greninja".into(),
            name_en: "Radiant Greninja".into(),
            set_code: "CS6.5C".into(),
            card_type: CardType::Pokemon,
            stage: Some(Stage::Basic), hp: Some(130), energy_type: Some(EnergyType::Water),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1), mechanic: Some(Mechanic::Radiant),
            attacks: vec![Attack { name: "Moonlight Shuriken".into(), cost: vec![EnergyType::Water, EnergyType::Water, EnergyType::Colorless], damage: 90, text: "90 to 2 Pokemon.".into(), effect_id: Some("attack_bench_snipe_double_90".into()) }],
            abilities: vec![Ability { name: "Concealed Cards".into(), text: "Discard Energy to draw 2.".into(), effect_id: "ability_concealed_cards".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "057"),
            name: "Zapdos".into(), name_en: "Zapdos".into(), set_code: "CS6aC".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(120), energy_type: Some(EnergyType::Lightning),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1), mechanic: None,
            attacks: vec![Attack { name: "Quick Strike".into(), cost: vec![EnergyType::Lightning, EnergyType::Colorless], damage: 70, text: "70 damage.".into(), effect_id: None }],
            abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "109"),
            name: "Flutter Mane".into(), name_en: "Flutter Mane".into(), set_code: "CSV7C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(110), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1), mechanic: None,
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "135"),
            name: "Fezandipiti ex".into(), name_en: "Fezandipiti ex".into(), set_code: "CSV8C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(220), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(2), mechanic: Some(Mechanic::Ex),
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "172"),
            name: "Bloodmoon Ursaluna ex".into(), name_en: "Bloodmoon Ursaluna ex".into(), set_code: "CSV8C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(220), energy_type: Some(EnergyType::Darkness),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Grass, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Psychic, multiplier: -30 }),
            retreat_cost: Some(2), mechanic: Some(Mechanic::Ex),
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("151C", "151"),
            name: "Mew ex".into(), name_en: "Mew ex".into(), set_code: "151C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(180), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1), mechanic: Some(Mechanic::Ex),
            attacks: vec![], abilities: vec![Ability { name: "Restart".into(), text: "Redraw hand.".into(), effect_id: "ability_restart".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "019"),
            name: "Raichu V".into(), name_en: "Raichu V".into(), set_code: "CS5aC".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(170), energy_type: Some(EnergyType::Lightning),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(2), mechanic: Some(Mechanic::V),
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS4DaC", "137"),
            name: "Raikou V".into(), name_en: "Raikou V".into(), set_code: "CS4DaC".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(170), energy_type: Some(EnergyType::Lightning),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1), mechanic: Some(Mechanic::V),
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5bC", "049"),
            name: "Lumineon V".into(), name_en: "Lumineon V".into(), set_code: "CS5bC".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(170), energy_type: Some(EnergyType::Water),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(2), mechanic: Some(Mechanic::V),
            attacks: vec![], abilities: vec![
                Ability { name: "Luminous Sign".into(), effect_id: "ability_luminous_sign".into(), text: "When played to bench, search a Supporter".into() }
            ], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "105"),
            name: "Squawkabilly ex".into(), name_en: "Squawkabilly ex".into(), set_code: "CSV2C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(240), energy_type: Some(EnergyType::Colorless),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(2), mechanic: Some(Mechanic::Ex),
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "065"),
            name: "Scream Tail".into(), name_en: "Scream Tail".into(), set_code: "CSV8C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(110), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(2), mechanic: None,
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "060"),
            name: "Klefki".into(), name_en: "Klefki".into(), set_code: "CSV1C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(70), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1), mechanic: None,
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "060"),
            name: "Drifloon".into(), name_en: "Drifloon".into(), set_code: "CSV2C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(80), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(2), mechanic: None,
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "094"),
            name: "Munkidori".into(), name_en: "Munkidori".into(), set_code: "CSV8C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(120), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(2), mechanic: None,
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "023"),
            name: "Rotom V".into(), name_en: "Rotom V".into(), set_code: "CSV6C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(160), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1), mechanic: Some(Mechanic::V),
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });

        // --- Pokemon: Charizard/Pidgeot deck allies ---
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "032"),
            name: "Duskull".into(), name_en: "Duskull".into(), set_code: "CS5.5C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(70), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(1), mechanic: None,
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "082"),
            name: "Dusclops".into(), name_en: "Dusclops".into(), set_code: "CSV8C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Stage1), hp: Some(120), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(3), mechanic: None,
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "083"),
            name: "Dusknoir".into(), name_en: "Dusknoir".into(), set_code: "CSV8C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Stage2), hp: Some(150), energy_type: Some(EnergyType::Psychic),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }),
            retreat_cost: Some(3), mechanic: None,
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "007"),
            name: "Radiant Charizard".into(), name_en: "Radiant Charizard".into(), set_code: "CS5.5C".into(),
            card_type: CardType::Pokemon, stage: Some(Stage::Basic), hp: Some(130), energy_type: Some(EnergyType::Fire),
            weakness: Some(TypeMultiplier { energy_type: EnergyType::Water, multiplier: 2 }),
            resistance: None, retreat_cost: Some(2), mechanic: Some(Mechanic::Radiant),
            attacks: vec![Attack { name: "Combustion Blast".into(), cost: vec![EnergyType::Fire, EnergyType::Fire, EnergyType::Colorless, EnergyType::Colorless], damage: 250, text: "250 if 1+ prizes remaining.".into(), effect_id: Some("attack_prize_condition_damage".into()) }],
            abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });

        // --- Trainer cards: Items ---
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "177"),
            name: "Buddy-Buddy Poffin".into(), name_en: "Buddy-Buddy Poffin".into(), set_code: "CSV7C".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "109"),
            name: "Super Rod".into(), name_en: "Super Rod".into(), set_code: "CSV1C".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "060"),
            name: "Earthen Vessel".into(), name_en: "Earthen Vessel".into(), set_code: "CSV6C".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "114"),
            name: "Counter Catcher".into(), name_en: "Counter Catcher".into(), set_code: "CSV6C".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "116"),
            name: "Switch Cart".into(), name_en: "Switch Cart".into(), set_code: "CS5DC".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6bC", "123"),
            name: "Lost Vacuum".into(), name_en: "Lost Vacuum".into(), set_code: "CSV6bC".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "060"),
            name: "Hisuian Heavy Ball".into(), name_en: "Hisuian Heavy Ball".into(), set_code: "CS5.5C".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "183"),
            name: "Night Stretcher".into(), name_en: "Night Stretcher".into(), set_code: "CSV8C".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "176"),
            name: "Secret Box".into(), name_en: "Secret Box".into(), set_code: "CSV8C".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "173"),
            name: "Unfair Stamp".into(), name_en: "Unfair Stamp".into(), set_code: "CSV8C".into(),
            card_type: CardType::Item, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });

        // --- Trainer cards: Supporters ---
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "191"),
            name: "Ciphermaniac's Codebreaking".into(), name_en: "Ciphermaniac's Codebreaking".into(), set_code: "CSV7C".into(),
            card_type: CardType::Supporter, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "140"),
            name: "Cyllene".into(), name_en: "Cyllene".into(), set_code: "CS5DC".into(),
            card_type: CardType::Supporter, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "125"),
            name: "Professor Turo's Scenario".into(), name_en: "Professor Turo's Scenario".into(), set_code: "CSV6C".into(),
            card_type: CardType::Supporter, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "127"),
            name: "Thorton".into(), name_en: "Thorton".into(), set_code: "CS6aC".into(),
            card_type: CardType::Supporter, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });

        // --- Tool cards ---
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "066"),
            name: "Forest Seal Stone".into(), name_en: "Forest Seal Stone".into(), set_code: "CSV7C".into(),
            card_type: CardType::Tool, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![
                Ability { name: "Star Alchemy".into(), effect_id: "tool_star_alchemy".into(), text: "Search deck for any 1 card".into() }
            ], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "188"),
            name: "Heavy Baton".into(), name_en: "Heavy Baton".into(), set_code: "CSV7C".into(),
            card_type: CardType::Tool, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None,
            damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "118"),
            name: "Bravery Charm".into(), name_en: "Bravery Charm".into(), set_code: "CSV1C".into(),
            card_type: CardType::Tool, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "185"),
            name: "Rescue Board".into(), name_en: "Rescue Board".into(), set_code: "CSV7C".into(),
            card_type: CardType::Tool, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "117"),
            name: "Defiance Band".into(), name_en: "Defiance Band".into(), set_code: "CSV1C".into(),
            card_type: CardType::Tool, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });

        // --- Stadium cards ---
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "201"),
            name: "Gravity Mountain".into(), name_en: "Gravity Mountain".into(), set_code: "CSV7C".into(),
            card_type: CardType::Stadium, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "071"),
            name: "Collapsed Stadium".into(), name_en: "Collapsed Stadium".into(), set_code: "CS6.5C".into(),
            card_type: CardType::Stadium, stage: None, hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });

        // ── Gouging Fire Ancient deck ──
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "051"), name: "Gouging Fire ex".into(), name_en: "Gouging Fire ex".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(230), energy_type: Some(EnergyType::Fire), weakness: Some(TypeMultiplier { energy_type: EnergyType::Water, multiplier: 2 }),
            resistance: None, retreat_cost: Some(2), mechanic: None,
            attacks: vec![
                Attack { name: "Burning Overrun".into(), cost: vec![EnergyType::Fire, EnergyType::Colorless], damage: 60, text: "".into(), effect_id: None },
                Attack { name: "Magma Blast".into(), cost: vec![EnergyType::Fire, EnergyType::Fire, EnergyType::Colorless, EnergyType::Colorless], damage: 220, text: "Discard 1 Energy".into(), effect_id: Some("attack_discard_energy_from_self".into()) },
            ], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS4DaC", "056"), name: "Entei V".into(), name_en: "Entei V".into(),
            set_code: "CS4DaC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220), energy_type: Some(EnergyType::Fire), weakness: Some(TypeMultiplier { energy_type: EnergyType::Water, multiplier: 2 }),
            resistance: None, retreat_cost: Some(2), mechanic: Some(Mechanic::V),
            attacks: vec![
                Attack { name: "Burning Arrow".into(), cost: vec![EnergyType::Fire, EnergyType::Colorless, EnergyType::Colorless], damage: 160, text: "".into(), effect_id: None },
            ], abilities: vec![
                Ability { name: "Burst Roar".into(), effect_id: "ability_burst_roar".into(), text: "Attach a Fire Energy from discard".into() },
            ], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "096"), name: "Roaring Moon ex".into(), name_en: "Roaring Moon ex".into(),
            set_code: "CSV6C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(230), energy_type: Some(EnergyType::Darkness), weakness: Some(TypeMultiplier { energy_type: EnergyType::Grass, multiplier: 2 }),
            resistance: None, retreat_cost: Some(2), mechanic: None,
            attacks: vec![
                Attack { name: "Calamity Storm".into(), cost: vec![EnergyType::Darkness, EnergyType::Darkness, EnergyType::Colorless], damage: 200, text: "".into(), effect_id: Some("attack_discard_stadium_bonus".into()) },
            ], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "006"), name: "Moltres".into(), name_en: "Moltres".into(),
            set_code: "CS5aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(120), energy_type: Some(EnergyType::Fire), weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: -30 }), retreat_cost: Some(1), mechanic: None,
            attacks: vec![
                Attack { name: "Flare".into(), cost: vec![EnergyType::Fire, EnergyType::Colorless], damage: 50, text: "".into(), effect_id: None },
            ], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "012"), name: "Delphox V".into(), name_en: "Delphox V".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(210), energy_type: Some(EnergyType::Fire), weakness: Some(TypeMultiplier { energy_type: EnergyType::Water, multiplier: 2 }),
            resistance: None, retreat_cost: Some(2), mechanic: Some(Mechanic::V),
            attacks: vec![
                Attack { name: "Magic Fire".into(), cost: vec![EnergyType::Fire, EnergyType::Colorless, EnergyType::Colorless], damage: 160, text: "".into(), effect_id: Some("attack_delphox_v_magic_fire".into()) },
            ], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        // Trainers
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "121"), name: "Professor Sada's Vitality".into(), name_en: "Professor Sada's Vitality".into(),
            set_code: "CSV6C".into(), card_type: CardType::Supporter, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "121"), name: "Professor's Research".into(), name_en: "Professor's Research".into(),
            set_code: "CSV1C".into(), card_type: CardType::Supporter, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "126"), name: "Dark Patch".into(), name_en: "Dark Patch".into(),
            set_code: "CS5DC".into(), card_type: CardType::Item, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1aC", "008"), name: "Energy Switch".into(), name_en: "Energy Switch".into(),
            set_code: "CSVH1aC".into(), card_type: CardType::Item, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "111"), name: "Pal Pad".into(), name_en: "Pal Pad".into(),
            set_code: "CSV1C".into(), card_type: CardType::Item, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        // Special Energy
        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "129"), name: "Jet Energy".into(), name_en: "Jet Energy".into(),
            set_code: "CSV4C".into(), card_type: CardType::SpecialEnergy, stage: None, hp: None, energy_type: Some(EnergyType::Colorless),
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: Some(vec![EnergyType::Colorless]), damage_modifier: None, retreat_modifier: Some(1),
        });
        // Stadium
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "152"), name: "Magma Basin".into(), name_en: "Magma Basin".into(),
            set_code: "CS5DC".into(), card_type: CardType::Stadium, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });

        // ── Future Box deck ──
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "033"), name: "Iron Leaves ex".into(), name_en: "Iron Leaves ex".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220), energy_type: Some(EnergyType::Grass), weakness: Some(TypeMultiplier { energy_type: EnergyType::Fire, multiplier: 2 }),
            resistance: None, retreat_cost: Some(1), mechanic: Some(Mechanic::Ex),
            attacks: vec![Attack { name: "Prism Blade".into(), cost: vec![EnergyType::Grass, EnergyType::Grass, EnergyType::Colorless], damage: 180, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "Quick Draw".into(), effect_id: "ability_quick_draw".into(), text: "When played to bench, switch with active and move energy".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "042"), name: "Iron Bundle".into(), name_en: "Iron Bundle".into(),
            set_code: "CSV6C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(100), energy_type: Some(EnergyType::Water), weakness: Some(TypeMultiplier { energy_type: EnergyType::Lightning, multiplier: 2 }),
            resistance: None, retreat_cost: Some(1), mechanic: None,
            attacks: vec![Attack { name: "Refrigerated Stream".into(), cost: vec![EnergyType::Water, EnergyType::Colorless, EnergyType::Colorless], damage: 80, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "Blower".into(), effect_id: "ability_iron_bundle_blower".into(), text: "If on bench, switch opponent active with bench".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "111"), name: "Iron Crown ex".into(), name_en: "Iron Crown ex".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220), energy_type: Some(EnergyType::Psychic), weakness: Some(TypeMultiplier { energy_type: EnergyType::Darkness, multiplier: 2 }),
            resistance: None, retreat_cost: Some(2), mechanic: Some(Mechanic::Ex),
            attacks: vec![Attack { name: "Double Blade".into(), cost: vec![EnergyType::Psychic, EnergyType::Colorless, EnergyType::Colorless], damage: 160, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "Azure Command".into(), effect_id: "ability_azure_command".into(), text: "Future Pokemon attacks +20 to opponent active".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "153"), name: "Miraidon".into(), name_en: "Miraidon".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(110), energy_type: Some(EnergyType::Lightning), weakness: Some(TypeMultiplier { energy_type: EnergyType::Fighting, multiplier: 2 }),
            resistance: None, retreat_cost: Some(2), mechanic: None,
            attacks: vec![
                Attack { name: "Peak Acceleration".into(), cost: vec![EnergyType::Colorless], damage: 40, text: "".into(), effect_id: None },
                Attack { name: "Electric Spark".into(), cost: vec![EnergyType::Lightning, EnergyType::Lightning, EnergyType::Psychic], damage: 160, text: "".into(), effect_id: None },
            ], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        // Trainers
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "180"), name: "Prime Catcher".into(), name_en: "Prime Catcher".into(),
            set_code: "CSV7C".into(), card_type: CardType::Item, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "119"), name: "Future Booster Energy Capsule".into(), name_en: "Future Booster Energy Capsule".into(),
            set_code: "CSV6C".into(), card_type: CardType::Tool, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: Some(0),
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "113"), name: "Pokegear 3.0".into(), name_en: "Pokegear 3.0".into(),
            set_code: "CSV2C".into(), card_type: CardType::Item, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "116"), name: "Techno Radar".into(), name_en: "Techno Radar".into(),
            set_code: "CSV6C".into(), card_type: CardType::Item, stage: None, hp: None, energy_type: None,
            weakness: None, resistance: None, retreat_cost: None, mechanic: None, attacks: vec![], abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });


// Auto-generated: 194 cards from PtcgDeckAgent JSON

        registry.register(CardDef {
            id: CardDefId::new("151C", "113"), name: "吉利蛋".into(), name_en: "Chansey".into(),
            set_code: "151C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(110u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "重掴".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 70u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "幸运奖励".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合，当从反面朝上的自己的奖赏卡中拿取了这张卡牌时，如果自己的备战区有空位的话，则在加入手牌前可以使用。将这只宝".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("151C", "132"), name: "百变怪".into(), name_en: "Ditto".into(),
            set_code: "151C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "粘粑粑".into(), cost: vec![EnergyType::Colorless], damage: 10u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "变身启动".into(), effect_id: "ability_placeholder".into(), text: "如果这只宝可梦在战斗场上的话，则仅在最初的自己的回合可以使用1次。选择自己牌库中的1张【基础】宝可梦（除「百变怪」外）。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CBB5C", "0301"), name: "三合一磁怪".into(), name_en: "".into(),
            set_code: "CBB5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(100u16), energy_type: Some(EnergyType::Lightning), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "雷电球".into(), cost: vec![EnergyType::Lightning, EnergyType::Colorless], damage: 40u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "过量放电".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次，如果使用了，则令这只宝可梦【昏厥】。选择自己弃牌区中最多3张基本能量，以任意方式附着于自己的【".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS4DaC", "137"), name: "雷公V".into(), name_en: "Raikou V".into(),
            set_code: "CS4DaC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(200u16), energy_type: Some(EnergyType::Lightning), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "雷电回旋曲".into(), cost: vec![EnergyType::Lightning, EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "瞬步".into(), effect_id: "ability_placeholder".into(), text: "如果这只宝可梦在战斗场上的话，则在自己的回合可以使用1次。从自己的牌库上方抽取1张卡牌。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS4DaC", "388"), name: "呐喊队的应援".into(), name_en: "Team Yell's Cheer".into(),
            set_code: "CS4DaC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "007"), name: "光辉喷火龙".into(), name_en: "Radiant Charizard".into(),
            set_code: "CS5.5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(160u16), energy_type: Some(EnergyType::Fire), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "炎爆".into(), cost: vec![EnergyType::Fire, EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 250u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "振奋之心".into(), effect_id: "ability_placeholder".into(), text: "这只宝可梦使用招式所需能量会减少与对手已经获得的奖赏卡张数相同数量的【无】能量。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "032"), name: "夜巡灵".into(), name_en: "Duskull".into(),
            set_code: "CS5.5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "迷惑".into(), cost: vec![EnergyType::Psychic], damage: 10u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "053"), name: "洗翠 黏美龙VSTAR".into(), name_en: "Hisuian Goodra VSTAR".into(),
            set_code: "CS5.5C".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(270u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "钢铁滚动".into(), cost: vec![EnergyType::Water, EnergyType::Metal, EnergyType::Colorless], damage: 200u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "润泽星耀".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用。将这只宝可梦的HP，全部回复。[对战中，己方的【VSTAR】力量只能使用1次。]".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "060"), name: "洗翠的沉重球".into(), name_en: "Hisuian Heavy Ball".into(),
            set_code: "CS5.5C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "064"), name: "黑连的关照".into(), name_en: "Cheren's Care".into(),
            set_code: "CS5.5C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "065"), name: "杜娟".into(), name_en: "Roxanne".into(),
            set_code: "CS5.5C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5.5C", "066"), name: "大嘴沼泽".into(), name_en: "Gapejaw Bog".into(),
            set_code: "CS5.5C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "085"), name: "达克莱伊V".into(), name_en: "Darkrai V".into(),
            set_code: "CS5DC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(210u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "暗之风".into(), cost: vec![EnergyType::Darkness, EnergyType::Colorless], damage: 50u16, text: "".into(), effect_id: None }, Attack { name: "暗黑洞".into(), cost: vec![EnergyType::Darkness, EnergyType::Darkness, EnergyType::Colorless], damage: 130u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "086"), name: "达克莱伊VSTAR".into(), name_en: "Darkrai VSTAR".into(),
            set_code: "CS5DC".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(270u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "恶之波动".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "星耀深渊".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用。选择自己弃牌区中最多2张物品，在给对手看过之后，加入手牌。[对战中，己方的VSTAR力量只能使用1".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "111"), name: "诡角鹿V".into(), name_en: "Wyrdeer V".into(),
            set_code: "CS5DC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "屏障猛攻".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 40u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "拓荒之路".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合，当这只宝可梦从备战区被放入战斗场时，可使用1次。选择任意数量的附着于自己场上宝可梦身上的能量，转附于这只宝".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "116"), name: "交替推车".into(), name_en: "Switch Cart".into(),
            set_code: "CS5DC".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "117"), name: "能量签".into(), name_en: "Energy Loto".into(),
            set_code: "CS5DC".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "138"), name: "珠贝".into(), name_en: "Irida".into(),
            set_code: "CS5DC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5DC", "140"), name: "星月".into(), name_en: "Cyllene".into(),
            set_code: "CS5DC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "019"), name: "雷丘V".into(), name_en: "Raichu V".into(),
            set_code: "CS5aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(200u16), energy_type: Some(EnergyType::Lightning), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "快速充能".into(), cost: vec![EnergyType::Lightning], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "强劲电光".into(), cost: vec![EnergyType::Lightning, EnergyType::Lightning], damage: 60u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "086"), name: "洗翠 大剑鬼VSTAR".into(), name_en: "Hisuian Samurott VSTAR".into(),
            set_code: "CS5aC".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(270u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "残忍之刃".into(), cost: vec![EnergyType::Darkness, EnergyType::Darkness], damage: 110u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "残月星耀".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用。给对手的1只宝可梦身上，放置4个伤害指示物。[对战中，己方的VSTAR力量只能使用1次。]".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "093"), name: "卡比兽".into(), name_en: "Snorlax".into(),
            set_code: "CS5aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(150u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(4u8), mechanic: None,
            attacks: vec![Attack { name: "瘫倒".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 150u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "挡道".into(), effect_id: "ability_placeholder".into(), text: "只要这只宝可梦在战斗场上，对手的战斗宝可梦，无法撤退。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "104"), name: "大牙狸".into(), name_en: "Bidoof".into(),
            set_code: "CS5aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "滚动".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "105"), name: "大尾狸".into(), name_en: "Bibarel".into(),
            set_code: "CS5aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(120u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "长尾粉碎".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 100u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "勤奋门牙".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。从牌库上方抽取卡牌，直到自己的手牌变为5张为止。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "107"), name: "阿尔宙斯VSTAR".into(), name_en: "Arceus VSTAR".into(),
            set_code: "CS5aC".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(280u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "三重新星".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 200u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "星耀诞生".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用。选择自己牌库中任意卡牌最多2张，加入手牌。并重洗牌库。[对战中，己方的VSTAR力量只能使用1次。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5aC", "113"), name: "清除古龙水".into(), name_en: "Canceling Cologne".into(),
            set_code: "CS5aC".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5bC", "045"), name: "巨翅飞鱼".into(), name_en: "Mantine".into(),
            set_code: "CS5bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(110u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "搬运上岸".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "水流利刃".into(), cost: vec![EnergyType::Water, EnergyType::Water, EnergyType::Colorless], damage: 100u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5bC", "051"), name: "起源帕路奇亚VSTAR".into(), name_en: "Origin Forme Palkia VSTAR".into(),
            set_code: "CS5bC".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(280u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "亚空潮漩".into(), cost: vec![EnergyType::Water, EnergyType::Water], damage: 60u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "星耀空扉".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用。选择自己弃牌区中最多3张【水】能量，以任意方式附着于自己的【水】宝可梦身上。[对战中，己方的VST".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5bC", "096"), name: "起源帝牙卢卡VSTAR".into(), name_en: "Origin Forme Dialga VSTAR".into(),
            set_code: "CS5bC".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(280u16), energy_type: Some(EnergyType::Metal), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "金属爆破".into(), cost: vec![EnergyType::Colorless], damage: 40u16, text: "".into(), effect_id: None }, Attack { name: "星耀时刻".into(), cost: vec![EnergyType::Metal, EnergyType::Metal, EnergyType::Metal, EnergyType::Metal, EnergyType::Colorless], damage: 220u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5bC", "111"), name: "大牙狸".into(), name_en: "Bidoof".into(),
            set_code: "CS5bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "终结门牙".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "毫不在意".into(), effect_id: "ability_placeholder".into(), text: "只要这只宝可梦，处于备战区，就不会受到招式的伤害。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5bC", "125"), name: "滨名的后援".into(), name_en: "Roseanne's Backup".into(),
            set_code: "CS5bC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS5bC", "128"), name: "神奥神殿".into(), name_en: "Temple of Sinnoh".into(),
            set_code: "CS5bC".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "029"), name: "拉鲁拉丝".into(), name_en: "Ralts".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "记忆跳越".into(), cost: vec![EnergyType::Psychic], damage: 10u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "032"), name: "勾魂眼".into(), name_en: "Sableye".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(80u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "抓".into(), cost: vec![EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }, Attack { name: "放逐矿脉".into(), cost: vec![EnergyType::Psychic], damage: 0u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "033"), name: "克雷色利亚".into(), name_en: "Cresselia".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(120u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "月光逆动".into(), cost: vec![EnergyType::Psychic], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "月光爆破".into(), cost: vec![EnergyType::Psychic, EnergyType::Psychic, EnergyType::Colorless], damage: 110u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "049"), name: "藏玛然特".into(), name_en: "Zamazenta".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(130u16), energy_type: Some(EnergyType::Metal), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "报仇".into(), cost: vec![EnergyType::Metal, EnergyType::Metal, EnergyType::Colorless], damage: 100u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "金属之盾".into(), effect_id: "ability_placeholder".into(), text: "如果这只宝可梦身上附有能量的话，则这只宝可梦所受到的招式的伤害「-30」。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "054"), name: "雷吉铎拉戈V".into(), name_en: "Regidrago V".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "天之呐喊".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "巨龙镭射".into(), cost: vec![EnergyType::Grass, EnergyType::Grass, EnergyType::Fire], damage: 130u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "055"), name: "雷吉铎拉戈VSTAR".into(), name_en: "Regidrago VSTAR".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(280u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "巨龙无双".into(), cost: vec![EnergyType::Grass, EnergyType::Grass, EnergyType::Fire], damage: 0u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "星耀遗产".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用。将自己牌库上方的7张卡牌放于弃牌区。然后，选择自己弃牌区中任意卡牌最多2张，在给对手看过之后，加入".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "063"), name: "健行鞋".into(), name_en: "Trekking Shoes".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "064"), name: "应急果冻".into(), name_en: "Emergency Jelly".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "066"), name: "森林封印石".into(), name_en: "Forest Seal Stone".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "070"), name: "阿渡".into(), name_en: "Lance".into(),
            set_code: "CS6.5C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6.5C", "072"), name: "V防守能量".into(), name_en: "V Guard Energy".into(),
            set_code: "CS6.5C".into(), card_type: CardType::SpecialEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "057"), name: "闪电鸟".into(), name_en: "Zapdos".into(),
            set_code: "CS6aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(120u16), energy_type: Some(EnergyType::Lightning), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "雷电球".into(), cost: vec![EnergyType::Lightning, EnergyType::Lightning, EnergyType::Colorless], damage: 110u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "电气象征".into(), effect_id: "ability_placeholder".into(), text: "只要这只宝可梦在场上，自己【雷】属性的【基础】宝可梦（除「闪电鸟」外）使用的招式，给对手战斗宝可梦造成的伤害「+10」。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "073"), name: "捷拉奥拉".into(), name_en: "Zeraora".into(),
            set_code: "CS6aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(110u16), energy_type: Some(EnergyType::Lightning), weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![Attack { name: "战斗利爪".into(), cost: vec![EnergyType::Lightning], damage: 30u16, text: "".into(), effect_id: None }, Attack { name: "音速伏特".into(), cost: vec![EnergyType::Lightning, EnergyType::Colorless, EnergyType::Colorless], damage: 80u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "083"), name: "铁哑铃".into(), name_en: "Beldum".into(),
            set_code: "CS6aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Metal), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "磁力抬升".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "撞击".into(), cost: vec![EnergyType::Metal, EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "102"), name: "洛奇亚V".into(), name_en: "Lugia V".into(),
            set_code: "CS6aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "读风".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "气旋俯冲".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 130u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "103"), name: "洛奇亚VSTAR".into(), name_en: "Lugia VSTAR".into(),
            set_code: "CS6aC".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(280u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "风暴俯冲".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 220u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "星耀汇聚".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用。选择自己弃牌区中最多2张【无】宝可梦（除「拥有规则的宝可梦」外），放于备战区。[对战中，己方的【V".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "113"), name: "始祖大鸟".into(), name_en: "Archeops".into(),
            set_code: "CS6aC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage2),
            hp: Some(150u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "高速之翼".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 120u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "原始涡轮".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。选择自己牌库中最多2张特殊能量，附着于自己的1只宝可梦身上。并重洗牌库。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "120"), name: "捕获香氛".into(), name_en: "Capturing Aroma".into(),
            set_code: "CS6aC".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "126"), name: "莎莉娜".into(), name_en: "Serena".into(),
            set_code: "CS6aC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "127"), name: "捩木".into(), name_en: "Thorton".into(),
            set_code: "CS6aC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "129"), name: "野贼三姐妹".into(), name_en: "Miss Fortune Sisters".into(),
            set_code: "CS6aC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6aC", "131"), name: "馈赠能量".into(), name_en: "Gift Energy".into(),
            set_code: "CS6aC".into(), card_type: CardType::SpecialEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "026"), name: "古月鸟".into(), name_en: "Cramorant".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(110u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "稀里鹕吐".into(), cost: vec![EnergyType::Water, EnergyType::Water, EnergyType::Colorless], damage: 110u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "放逐供应".into(), effect_id: "ability_placeholder".into(), text: "如果自己放逐区有4张以上（包含4张）卡牌的话，则这只宝可梦使用招式所需能量，全部消除。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "028"), name: "光辉胡地".into(), name_en: "Radiant Alakazam".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(130u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "意志控制者".into(), cost: vec![EnergyType::Psychic, EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "伤痛汤匙".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。选择对手场上1只宝可梦身上放置的最多2个伤害指示物，转放于对手1只其他宝可梦身上。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "052"), name: "花疗环环".into(), name_en: "Comfey".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "回转攻击".into(), cost: vec![EnergyType::Psychic, EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "选花".into(), effect_id: "ability_placeholder".into(), text: "如果这只宝可梦在战斗场上的话，则在自己的回合可以使用1次。查看自己牌库上方2张卡牌，选择其中1张卡牌，加入手牌。将剩余的".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "058"), name: "多龙巴鲁托".into(), name_en: "Dragapult".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage2),
            hp: Some(150u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "龙之发射器".into(), cost: vec![EnergyType::Psychic], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "幽魂射击".into(), cost: vec![EnergyType::Psychic, EnergyType::Colorless], damage: 120u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "107"), name: "骑拉帝纳V".into(), name_en: "Giratina V".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "深渊探求".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "撕裂".into(), cost: vec![EnergyType::Grass, EnergyType::Psychic, EnergyType::Colorless], damage: 160u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "108"), name: "骑拉帝纳VSTAR".into(), name_en: "Giratina VSTAR".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: None,
            hp: Some(280u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "放逐冲击".into(), cost: vec![EnergyType::Grass, EnergyType::Psychic, EnergyType::Colorless], damage: 280u16, text: "".into(), effect_id: None }, Attack { name: "星耀安魂曲".into(), cost: vec![EnergyType::Grass, EnergyType::Psychic], damage: 0u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "112"), name: "大比鸟V".into(), name_en: "Pidgeot V".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(210u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "乘风飞翔".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 80u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "消失之翼".into(), effect_id: "ability_placeholder".into(), text: "如果这只宝可梦在备战区的话，则在自己的回合可以使用1次。将这只宝可梦，以及放于其身上的所有卡牌，放回自己的牌库并重洗牌库".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "113"), name: "卡比兽".into(), name_en: "Snorlax".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(150u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(4u8), mechanic: None,
            attacks: vec![Attack { name: "轰隆鼾声".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 180u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "无畏脂肪".into(), effect_id: "ability_placeholder".into(), text: "这只宝可梦，不会受到对手宝可梦所使用招式的效果影响。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "117"), name: "泡沫栗鼠".into(), name_en: "Minccino".into(),
            set_code: "CS6bC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "呼朋引伴".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "拍击".into(), cost: vec![EnergyType::Colorless], damage: 10u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "122"), name: "幻象之门".into(), name_en: "Mirage Gate".into(),
            set_code: "CS6bC".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "123"), name: "放逐吸尘器".into(), name_en: "Lost Vacuum".into(),
            set_code: "CS6bC".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "125"), name: "阿克罗玛的实验".into(), name_en: "Colress's Experiment".into(),
            set_code: "CS6bC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CS6bC", "130"), name: "放逐市".into(), name_en: "Lost City".into(),
            set_code: "CS6bC".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSNC", "003"), name: "起源帕路奇亚V".into(), name_en: "Origin Forme Palkia V".into(),
            set_code: "CSNC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "领域支配".into(), cost: vec![EnergyType::Water], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "水炮破坏".into(), cost: vec![EnergyType::Water, EnergyType::Water, EnergyType::Colorless], damage: 200u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSNC", "007"), name: "洗翠 大剑鬼V".into(), name_en: "Hisuian Samurott V".into(),
            set_code: "CSNC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "深青坠击".into(), cost: vec![EnergyType::Darkness], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "暗影之刃".into(), cost: vec![EnergyType::Darkness, EnergyType::Darkness, EnergyType::Darkness], damage: 180u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSNC", "008"), name: "起源帝牙卢卡V".into(), name_en: "Origin Forme Dialga V".into(),
            set_code: "CSNC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Metal), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "金属涂层".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "时间断绝".into(), cost: vec![EnergyType::Metal, EnergyType::Metal, EnergyType::Metal, EnergyType::Colorless], damage: 180u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSNC", "009"), name: "阿尔宙斯V".into(), name_en: "Arceus V".into(),
            set_code: "CSNC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "三重蓄能".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "力量利刃".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 130u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "042"), name: "小磁怪".into(), name_en: "Magnemite".into(),
            set_code: "CSV1C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Lightning), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "互斥".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "电球".into(), cost: vec![EnergyType::Lightning], damage: 10u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "053"), name: "怨影娃娃".into(), name_en: "Shuppet".into(),
            set_code: "CSV1C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "阴影包围".into(), cost: vec![EnergyType::Psychic], damage: 10u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "054"), name: "诅咒娃娃ex".into(), name_en: "Banette ex".into(),
            set_code: "CSV1C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(250u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "暗夜难明".into(), cost: vec![EnergyType::Psychic], damage: 30u16, text: "".into(), effect_id: None }, Attack { name: "灵骚".into(), cost: vec![EnergyType::Psychic, EnergyType::Colorless], damage: 60u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "055"), name: "花岩怪".into(), name_en: "Spiritomb".into(),
            set_code: "CSV1C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "瞬间消失".into(), cost: vec![EnergyType::Colorless], damage: 10u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "漆黑灾祸".into(), effect_id: "ability_placeholder".into(), text: "只要这只宝可梦在场上，双方场上【基础】宝可梦的「宝可梦【V】」的特性，全部消除。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "060"), name: "钥圈儿".into(), name_en: "Klefki".into(),
            set_code: "CSV1C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "狙落".into(), cost: vec![EnergyType::Colorless], damage: 10u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "恶作剧之锁".into(), effect_id: "ability_placeholder".into(), text: "只要这只宝可梦在战斗场上，双方场上【基础】宝可梦的特性（除「恶作剧之锁」外），全部消除。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "079"), name: "摔角鹰人".into(), name_en: "Hawlucha".into(),
            set_code: "CSV1C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Fighting), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "翅膀攻击".into(), cost: vec![EnergyType::Fighting, EnergyType::Colorless, EnergyType::Colorless], damage: 70u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "飞身入场".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合，当将这张卡牌从手牌使出放于备战区时，可使用1次。给对手的2只备战宝可梦身上，各放置1个伤害指示物。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "099"), name: "贪心栗鼠".into(), name_en: "Skwovet".into(),
            set_code: "CSV1C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "咬住".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "巢穴藏身".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。将自己所有的手牌翻到反面重洗，放回牌库下方。然后，从牌库上方抽取1张卡牌。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "108"), name: "粉碎之锤".into(), name_en: "Crushing Hammer".into(),
            set_code: "CSV1C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "115"), name: "学习装置".into(), name_en: "Exp. Share".into(),
            set_code: "CSV1C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "116"), name: "讲究腰带".into(), name_en: "Choice Belt".into(),
            set_code: "CSV1C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "117"), name: "不服输头带".into(), name_en: "Defiance Band".into(),
            set_code: "CSV1C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "120"), name: "吉尼亚".into(), name_en: "Jacq".into(),
            set_code: "CSV1C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "123"), name: "派帕".into(), name_en: "Arven".into(),
            set_code: "CSV1C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "124"), name: "牡丹".into(), name_en: "Penny".into(),
            set_code: "CSV1C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "126"), name: "桌台市".into(), name_en: "Mesagoza".into(),
            set_code: "CSV1C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV1C", "127"), name: "夜光能量".into(), name_en: "Luminous Energy".into(),
            set_code: "CSV1C".into(), card_type: CardType::SpecialEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "028"), name: "呱呱泡蛙".into(), name_en: "Froakie".into(),
            set_code: "CSV2C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "跳一下".into(), cost: vec![EnergyType::Water], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "060"), name: "飘飘球".into(), name_en: "Drifloon".into(),
            set_code: "CSV2C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "起风".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 10u16, text: "".into(), effect_id: None }, Attack { name: "气球炸弹".into(), cost: vec![EnergyType::Psychic, EnergyType::Psychic], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "111"), name: "鼓励信".into(), name_en: "Letter of Encouragement".into(),
            set_code: "CSV2C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "118"), name: "莉佳的邀请".into(), name_en: "Erika's Invitation".into(),
            set_code: "CSV2C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "126"), name: "月光之丘".into(), name_en: "Moonlit Hill".into(),
            set_code: "CSV2C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV2C", "127"), name: "深钵镇".into(), name_en: "Artazon".into(),
            set_code: "CSV2C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "031"), name: "古玉鱼ex".into(), name_en: "Chi-Yu ex".into(),
            set_code: "CSV3C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(190u16), energy_type: Some(EnergyType::Fire), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "妒火中烧".into(), cost: vec![EnergyType::Fire], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "火焰巨浪".into(), cost: vec![EnergyType::Fire, EnergyType::Fire], damage: 100u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "040"), name: "凉脊龙".into(), name_en: "Frigibax".into(),
            set_code: "CSV3C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "撞击".into(), cost: vec![EnergyType::Water, EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "042"), name: "戟脊龙".into(), name_en: "Baxcalibur".into(),
            set_code: "CSV3C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage2),
            hp: Some(160u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "爆破之尾".into(), cost: vec![EnergyType::Water, EnergyType::Water, EnergyType::Colorless], damage: 130u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "极低温".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用任意次。选择自己手牌中的1张「基本【水】能量」，附着于自己的宝可梦身上。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "043"), name: "古剑豹ex".into(), name_en: "Chien-Pao ex".into(),
            set_code: "CSV3C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(220u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "冰雹利刃".into(), cost: vec![EnergyType::Water, EnergyType::Water], damage: 60u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "战栗冷气".into(), effect_id: "ability_placeholder".into(), text: "如果这只宝可梦在战斗场上的话，则在自己的回合可以使用1次。选择自己牌库中最多2张「基本【水】能量」，在给对手看过之后，加".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "062"), name: "谜拟丘".into(), name_en: "Mimikyu".into(),
            set_code: "CSV3C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "幽灵之眼".into(), cost: vec![EnergyType::Psychic, EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "神秘守护".into(), effect_id: "ability_placeholder".into(), text: "这只宝可梦，不受到对手「宝可梦【ex】・【V】」的招式的伤害。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "115"), name: "超级能量回收".into(), name_en: "Superior Energy Retrieval".into(),
            set_code: "CSV3C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "116"), name: "超级球".into(), name_en: "Great Ball".into(),
            set_code: "CSV3C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "125"), name: "皮拿".into(), name_en: "Giacomo".into(),
            set_code: "CSV3C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV3C", "130"), name: "治疗能量".into(), name_en: "Therapeutic Energy".into(),
            set_code: "CSV3C".into(), card_type: CardType::SpecialEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "044"), name: "皮宝宝".into(), name_en: "Cleffa".into(),
            set_code: "CSV4C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(30u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![Attack { name: "握握抽取".into(), cost: vec![], damage: 0u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "063"), name: "索财灵".into(), name_en: "Gimmighoul".into(),
            set_code: "CSV4C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "连掷硬币".into(), cost: vec![EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "089"), name: "赛富豪ex".into(), name_en: "Gholdengo ex".into(),
            set_code: "CSV4C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(260u16), energy_type: Some(EnergyType::Metal), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "淘金潮".into(), cost: vec![EnergyType::Metal], damage: 50u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "嘉奖硬币".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。从自己牌库上方抽取1张卡牌。如果这只宝可梦在战斗场上的话，则额外抽取1张卡牌。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "099"), name: "波波".into(), name_en: "Pidgey".into(),
            set_code: "CSV4C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "起风".into(), cost: vec![EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "117"), name: "豪华斗篷".into(), name_en: "Luxurious Cape".into(),
            set_code: "CSV4C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV4C", "119"), name: "招式学习器 能量涡轮".into(), name_en: "Technical Machine: Turbo Energize".into(),
            set_code: "CSV4C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "014"), name: "小火龙".into(), name_en: "Charmander".into(),
            set_code: "CSV5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Fire), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "高温冲撞".into(), cost: vec![EnergyType::Fire], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "075"), name: "喷火龙ex".into(), name_en: "Charizard ex".into(),
            set_code: "CSV5C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage2),
            hp: Some(330u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "燃烧黑暗".into(), cost: vec![EnergyType::Fire, EnergyType::Fire], damage: 180u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "烈炎支配".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合，当将这张卡牌从手牌使出并进行进化时，可使用1次。选择自己牌库中最多3张「基本【火】能量」，以任意方式附着于".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "114"), name: "能量贴纸".into(), name_en: "Energy Sticker".into(),
            set_code: "CSV5C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "119"), name: "招式学习器 进化".into(), name_en: "Technical Machine: Evolution".into(),
            set_code: "CSV5C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "120"), name: "招式学习器 退化".into(), name_en: "Technical Machine: Devolution".into(),
            set_code: "CSV5C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "127"), name: "梅洛可".into(), name_en: "Mela".into(),
            set_code: "CSV5C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV5C", "128"), name: "城镇百货".into(), name_en: "Town Store".into(),
            set_code: "CSV5C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "065"), name: "吼叫尾".into(), name_en: "Scream Tail".into(),
            set_code: "CSV6C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(90u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "巴掌".into(), cost: vec![EnergyType::Psychic], damage: 30u16, text: "".into(), effect_id: None }, Attack { name: "凶暴吼叫".into(), cost: vec![EnergyType::Psychic, EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "082"), name: "爬地翅".into(), name_en: "Slither Wing".into(),
            set_code: "CSV6C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(140u16), energy_type: Some(EnergyType::Fighting), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "踏平".into(), cost: vec![EnergyType::Fighting], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "烫伤怒涛".into(), cost: vec![EnergyType::Fighting, EnergyType::Fighting], damage: 120u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "115"), name: "大地容器".into(), name_en: "Earthen Vessel".into(),
            set_code: "CSV6C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "125"), name: "弗图博士的剧本".into(), name_en: "Professor Turo's Scenario".into(),
            set_code: "CSV6C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "127"), name: "自行车道".into(), name_en: "Cycling Road".into(),
            set_code: "CSV6C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV6C", "128"), name: "宝可梦联盟总部".into(), name_en: "Pokémon League Headquarters".into(),
            set_code: "CSV6C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "036"), name: "火稚鸡".into(), name_en: "Torchic".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Fire), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "抓".into(), cost: vec![EnergyType::Fire, EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "038"), name: "火焰鸡ex".into(), name_en: "Blaziken ex".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage2),
            hp: Some(320u16), energy_type: Some(EnergyType::Fire), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "燃烧旋踢".into(), cost: vec![EnergyType::Fire, EnergyType::Colorless], damage: 200u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "沸腾斗志".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。选择自己弃牌区中的1张基本能量，附着于自己的宝可梦身上。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "091"), name: "铁荆棘ex".into(), name_en: "Iron Thorns ex".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(230u16), energy_type: Some(EnergyType::Lightning), weakness: None, resistance: None,
            retreat_cost: Some(4u8), mechanic: None,
            attacks: vec![Attack { name: "伏特旋风".into(), cost: vec![EnergyType::Lightning, EnergyType::Colorless, EnergyType::Colorless], damage: 140u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "初始化".into(), effect_id: "ability_placeholder".into(), text: "只要这只宝可梦在战斗场上，双方场上的「拥有规则的宝可梦」（除「未来」宝可梦外）的特性，全部消除。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "132"), name: "沙铁皮".into(), name_en: "Sandy Shocks".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(120u16), energy_type: Some(EnergyType::Fighting), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "磁场炸裂".into(), cost: vec![EnergyType::Fighting], damage: 20u16, text: "".into(), effect_id: None }, Attack { name: "力量宝石".into(), cost: vec![EnergyType::Fighting, EnergyType::Colorless], damage: 60u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "141"), name: "奇麒麟ex".into(), name_en: "Farigiraf ex".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(260u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "恶劣光束".into(), cost: vec![EnergyType::Psychic, EnergyType::Colorless, EnergyType::Colorless], damage: 160u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "尾甲".into(), effect_id: "ability_placeholder".into(), text: "这只宝可梦，不会受到对手【基础】宝可梦的「宝可梦【ex】」的招式的伤害。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "143"), name: "轰鸣月".into(), name_en: "Roaring Moon".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(140u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "报仇箭羽".into(), cost: vec![EnergyType::Darkness, EnergyType::Darkness], damage: 70u16, text: "".into(), effect_id: None }, Attack { name: "高速之翼".into(), cost: vec![EnergyType::Darkness, EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 120u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "147"), name: "金属怪".into(), name_en: "Metang".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(100u16), energy_type: Some(EnergyType::Metal), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "光束".into(), cost: vec![EnergyType::Metal, EnergyType::Colorless, EnergyType::Colorless], damage: 60u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "金属制造者".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。查看自己牌库上方4张卡牌，选择其中任意数量的「基本【钢】能量」，以任意方式附着于自己的宝可梦身".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "154"), name: "猛雷鼓ex".into(), name_en: "Raging Bolt ex".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(240u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "飞溅咆哮".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "极雷轰".into(), cost: vec![EnergyType::Lightning, EnergyType::Fighting], damage: 70u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "161"), name: "土龙弟弟".into(), name_en: "Dunsparce".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![Attack { name: "啃咬".into(), cost: vec![EnergyType::Colorless], damage: 10u16, text: "".into(), effect_id: None }, Attack { name: "挖洞".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "162"), name: "土龙节节".into(), name_en: "Dudunsparce".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(140u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "大地粉碎".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 90u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "逃跑抽取".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。从自己牌库上方抽取3张卡牌。然后，将这只宝可梦，以及放于其身上的所有卡牌，放回牌库并重洗牌库。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "170"), name: "泡沫栗鼠".into(), name_en: "Minccino".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "敲打".into(), cost: vec![EnergyType::Colorless], damage: 10u16, text: "".into(), effect_id: None }, Attack { name: "扫除".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "171"), name: "奇诺栗鼠".into(), name_en: "Cinccino".into(),
            set_code: "CSV7C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(110u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "重掴".into(), cost: vec![EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }, Attack { name: "特殊滚动".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 70u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "175"), name: "改造之锤".into(), name_en: "Enhanced Hammer".into(),
            set_code: "CSV7C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "177"), name: "友好宝芬".into(), name_en: "Buddy-Buddy Poffin".into(),
            set_code: "CSV7C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "178"), name: "高级香氛".into(), name_en: "Hyper Aroma".into(),
            set_code: "CSV7C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "181"), name: "大师球".into(), name_en: "Master Ball".into(),
            set_code: "CSV7C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "187"), name: "英雄斗篷".into(), name_en: "Hero's Cape".into(),
            set_code: "CSV7C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "189"), name: "极限腰带".into(), name_en: "Maximum Belt".into(),
            set_code: "CSV7C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "191"), name: "暗码迷的解读".into(), name_en: "Ciphermaniac's Codebreaking".into(),
            set_code: "CSV7C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "194"), name: "赛吉".into(), name_en: "Salvatore".into(),
            set_code: "CSV7C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "197"), name: "枇琶".into(), name_en: "Eri".into(),
            set_code: "CSV7C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "202"), name: "全金属实验室".into(), name_en: "Full Metal Lab".into(),
            set_code: "CSV7C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV7C", "204"), name: "薄雾能量".into(), name_en: "Mist Energy".into(),
            set_code: "CSV7C".into(), card_type: CardType::SpecialEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "028"), name: "厄诡椪 碧草面具ex".into(), name_en: "Teal Mask Ogerpon ex".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(210u16), energy_type: Some(EnergyType::Grass), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "万叶阵雨".into(), cost: vec![EnergyType::Grass, EnergyType::Grass, EnergyType::Grass], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "碧草之舞".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。选择自己手牌中的1张「基本【草】能量」，附着于这只宝可梦身上。然后，从自己牌库上方抽取1张卡牌".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "067"), name: "厄诡椪 水井面具ex".into(), name_en: "Wellspring Mask Ogerpon ex".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(210u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "啜泣".into(), cost: vec![EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }, Attack { name: "激流水泵".into(), cost: vec![EnergyType::Water, EnergyType::Colorless, EnergyType::Colorless], damage: 100u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "078"), name: "麒麟奇".into(), name_en: "Girafarig".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(100u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "双向头击".into(), cost: vec![EnergyType::Colorless], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "081"), name: "夜巡灵".into(), name_en: "Duskull".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(60u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "渡魂".into(), cost: vec![EnergyType::Psychic], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "喃喃自语".into(), cost: vec![EnergyType::Psychic, EnergyType::Psychic], damage: 30u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "082"), name: "彷徨夜灵".into(), name_en: "Dusclops".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(90u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "磷火".into(), cost: vec![EnergyType::Psychic, EnergyType::Psychic], damage: 50u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "咒怨炸弹".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次，如果使用了，则令这只宝可梦【昏厥】。给对手的1只宝可梦身上，放置5个伤害指示物。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "083"), name: "黑夜魔灵".into(), name_en: "Dusknoir".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage2),
            hp: Some(160u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "影子束缚".into(), cost: vec![EnergyType::Psychic, EnergyType::Psychic, EnergyType::Colorless], damage: 150u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "咒怨炸弹".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次，如果使用了，则令这只宝可梦【昏厥】。给对手的1只宝可梦身上，放置13个伤害指示物。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "121"), name: "厄诡椪 础石面具ex".into(), name_en: "Cornerstone Mask Ogerpon ex".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(210u16), energy_type: Some(EnergyType::Fighting), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "打爆".into(), cost: vec![EnergyType::Fighting, EnergyType::Colorless, EnergyType::Colorless], damage: 140u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "础石之姿".into(), effect_id: "ability_placeholder".into(), text: "这只宝可梦，不受到对手拥有特性的宝可梦的招式的伤害。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "133"), name: "够赞狗ex".into(), name_en: "Okidogi ex".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(250u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(3u8), mechanic: None,
            attacks: vec![Attack { name: "毒液肌力".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "狂热锁链".into(), cost: vec![EnergyType::Darkness, EnergyType::Darkness, EnergyType::Colorless], damage: 130u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "136"), name: "桃歹郎ex".into(), name_en: "Pecharunt ex".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(190u16), energy_type: Some(EnergyType::Darkness), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "焦躁爆破".into(), cost: vec![EnergyType::Darkness, EnergyType::Darkness], damage: 60u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "支配锁链".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。选择自己备战区中的1只【恶】宝可梦（除「桃歹郎【ex】」外），将其与战斗宝可梦互换。然后，令新".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "153"), name: "双斧战龙".into(), name_en: "Haxorus".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage2),
            hp: Some(170u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "巨斧劈落".into(), cost: vec![EnergyType::Fighting], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "龙之波动".into(), cost: vec![EnergyType::Fighting, EnergyType::Metal], damage: 230u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "157"), name: "多龙梅西亚".into(), name_en: "Dreepy".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "小哀怨".into(), cost: vec![EnergyType::Psychic], damage: 10u16, text: "".into(), effect_id: None }, Attack { name: "咬住".into(), cost: vec![EnergyType::Fire, EnergyType::Psychic], damage: 40u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "158"), name: "多龙奇".into(), name_en: "Drakloak".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(90u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "龙之头击".into(), cost: vec![EnergyType::Fire, EnergyType::Psychic], damage: 70u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "侦察指令".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。查看自己牌库上方2张卡牌，选择其中1张卡牌，加入手牌。将剩余的卡牌，放回牌库下方。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "159"), name: "多龙巴鲁托ex".into(), name_en: "Dragapult ex".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage2),
            hp: Some(320u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "喷射头击".into(), cost: vec![EnergyType::Colorless], damage: 70u16, text: "".into(), effect_id: None }, Attack { name: "幻影潜袭".into(), cost: vec![EnergyType::Fire, EnergyType::Psychic], damage: 200u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "160"), name: "米立龙".into(), name_en: "Tatsugiri".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Dragon), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "冲浪".into(), cost: vec![EnergyType::Fire, EnergyType::Water], damage: 50u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "揽客".into(), effect_id: "ability_placeholder".into(), text: "如果这只宝可梦在战斗场上的话，则在自己的回合可以使用1次。查看自己牌库上方6张卡牌，选择其中1张支援者，在给对手看过之后".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "164"), name: "吉利蛋".into(), name_en: "Chansey".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(120u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "幸运附着".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "潜力".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 80u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "165"), name: "幸福蛋ex".into(), name_en: "Blissey ex".into(),
            set_code: "CSV8C".into(), card_type: CardType::Pokemon, stage: Some(Stage::Stage1),
            hp: Some(300u16), energy_type: Some(EnergyType::Colorless), weakness: None, resistance: None,
            retreat_cost: Some(4u8), mechanic: None,
            attacks: vec![Attack { name: "报恩".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless, EnergyType::Colorless], damage: 180u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "幸福切换".into(), effect_id: "ability_placeholder".into(), text: "在自己的回合可以使用1次。选择自己场上宝可梦身上附着的1个基本能量，转附于自己其他宝可梦身上。".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "175"), name: "配乐之笛".into(), name_en: "Accompanying Flute".into(),
            set_code: "CSV8C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "177"), name: "黑暗球".into(), name_en: "Dusk Ball".into(),
            set_code: "CSV8C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "186"), name: "璀璨结晶".into(), name_en: "Sparkling Crystal".into(),
            set_code: "CSV8C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "187"), name: "锁链粘糕".into(), name_en: "Binding Mochi".into(),
            set_code: "CSV8C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "190"), name: "手持循环扇".into(), name_en: "Handheld Fan".into(),
            set_code: "CSV8C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "191"), name: "阿克罗玛的执念".into(), name_en: "Colress's Tenacity".into(),
            set_code: "CSV8C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "192"), name: "阿杏的秘招".into(), name_en: "Janine's Secret Art".into(),
            set_code: "CSV8C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "195"), name: "库瑟洛斯奇的企图".into(), name_en: "Xerosic's Machinations".into(),
            set_code: "CSV8C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "198"), name: "乌栗".into(), name_en: "Kieran".into(),
            set_code: "CSV8C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "199"), name: "丹瑜".into(), name_en: "Carmine".into(),
            set_code: "CSV8C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "203"), name: "阻碍之塔".into(), name_en: "Jamming Tower".into(),
            set_code: "CSV8C".into(), card_type: CardType::Stadium, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSV8C", "207"), name: "遗赠能量".into(), name_en: "Legacy Energy".into(),
            set_code: "CSV8C".into(), card_type: CardType::SpecialEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "DAR"), name: "基本恶能量".into(), name_en: "Darkness Energy".into(),
            set_code: "CSVE1C".into(), card_type: CardType::BasicEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "FIG"), name: "基本斗能量".into(), name_en: "Fighting Energy".into(),
            set_code: "CSVE1C".into(), card_type: CardType::BasicEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "FIR"), name: "基本火能量".into(), name_en: "Fire Energy".into(),
            set_code: "CSVE1C".into(), card_type: CardType::BasicEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "GRA"), name: "基本草能量".into(), name_en: "Grass Energy".into(),
            set_code: "CSVE1C".into(), card_type: CardType::BasicEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "LIG"), name: "基本雷能量".into(), name_en: "Lightning Energy".into(),
            set_code: "CSVE1C".into(), card_type: CardType::BasicEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "MET"), name: "基本钢能量".into(), name_en: "Metal Energy".into(),
            set_code: "CSVE1C".into(), card_type: CardType::BasicEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "PSY"), name: "基本超能量".into(), name_en: "Psychic Energy".into(),
            set_code: "CSVE1C".into(), card_type: CardType::BasicEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVE1C", "WAT"), name: "基本水能量".into(), name_en: "Water Energy".into(),
            set_code: "CSVE1C".into(), card_type: CardType::BasicEnergy, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1C", "034"), name: "能量回收".into(), name_en: "Energy Retrieval".into(),
            set_code: "CSVH1C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1C", "035"), name: "能量输送".into(), name_en: "Energy Search".into(),
            set_code: "CSVH1C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1C", "043"), name: "巢穴球".into(), name_en: "Nest Ball".into(),
            set_code: "CSVH1C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1C", "045"), name: "神奇糖果".into(), name_en: "Rare Candy".into(),
            set_code: "CSVH1C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1C", "047"), name: "宝可梦捕捉器".into(), name_en: "Pokémon Catcher".into(),
            set_code: "CSVH1C".into(), card_type: CardType::Item, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1C", "050"), name: "活力头带".into(), name_en: "Vitality Band".into(),
            set_code: "CSVH1C".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1C", "051"), name: "裁判".into(), name_en: "Judge".into(),
            set_code: "CSVH1C".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH1aC", "023"), name: "老大的指令".into(), name_en: "Boss's Orders".into(),
            set_code: "CSVH1aC".into(), card_type: CardType::Supporter, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH3aC", "017"), name: "招式学习器 临危一击".into(), name_en: "Technical Machine: Crisis Punch".into(),
            set_code: "CSVH3aC".into(), card_type: CardType::Tool, stage: None,
            hp: None, energy_type: None, weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("CSVH3pC", "001"), name: "凉脊龙".into(), name_en: "Frigibax".into(),
            set_code: "CSVH3pC".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: Some(2u8), mechanic: None,
            attacks: vec![Attack { name: "招来".into(), cost: vec![EnergyType::Water], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "敲打".into(), cost: vec![EnergyType::Water, EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("SVP", "067"), name: "小火龙".into(), name_en: "Charmander".into(),
            set_code: "SVP".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(70u16), energy_type: Some(EnergyType::Fire), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "火花".into(), cost: vec![EnergyType::Fire, EnergyType::Fire], damage: 40u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("SVP", "102"), name: "小磁怪".into(), name_en: "Magnemite".into(),
            set_code: "SVP".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(50u16), energy_type: Some(EnergyType::Lightning), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "磁力充能".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "高速球".into(), cost: vec![EnergyType::Lightning, EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("SVP", "105"), name: "索财灵".into(), name_en: "Gimmighoul".into(),
            set_code: "SVP".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(50u16), energy_type: Some(EnergyType::Psychic), weakness: None, resistance: None,
            retreat_cost: Some(1u8), mechanic: None,
            attacks: vec![Attack { name: "呼朋引伴".into(), cost: vec![EnergyType::Colorless], damage: 0u16, text: "".into(), effect_id: None }, Attack { name: "推打".into(), cost: vec![EnergyType::Colorless, EnergyType::Colorless], damage: 20u16, text: "".into(), effect_id: None }],
            abilities: vec![],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });
        registry.register(CardDef {
            id: CardDefId::new("UTEST", "001"), name: "Dynamic Registration".into(), name_en: "".into(),
            set_code: "UTEST".into(), card_type: CardType::Pokemon, stage: Some(Stage::Basic),
            hp: Some(120u16), energy_type: Some(EnergyType::Water), weakness: None, resistance: None,
            retreat_cost: None, mechanic: None,
            attacks: vec![Attack { name: "鐐庣垎".into(), cost: vec![EnergyType::Fire, EnergyType::Fire], damage: 100u16, text: "".into(), effect_id: None }],
            abilities: vec![Ability { name: "娴姳姘村笜".into(), effect_id: "ability_placeholder".into(), text: "".into() }],
            provides_energy: None, damage_modifier: None, retreat_modifier: None,
        });

// End auto-generated (194 cards)
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_def_id() {
        let id = CardDefId::new("CSV1C", "050");
        assert_eq!(id.0, "CSV1C_050");
    }

    #[test]
    fn test_energy_type_conversion() {
        assert_eq!(EnergyType::from_char('R'), Some(EnergyType::Fire));
        assert_eq!(EnergyType::from_char('L'), Some(EnergyType::Lightning));
        assert_eq!(EnergyType::to_char(&EnergyType::Fire), 'R');
    }

    #[test]
    fn test_card_registry() {
        let registry = presets::load_miraidon_charizard_cards();
        assert!(registry.len() > 0);
        
        let miraidon = registry.get(&CardDefId::new("CSV1C", "050"));
        assert!(miraidon.is_some());
        assert!(miraidon.unwrap().is_ex());
    }
}