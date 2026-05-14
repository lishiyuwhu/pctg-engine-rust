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
            attacks: vec![], abilities: vec![], provides_energy: None, damage_modifier: None, retreat_modifier: None,
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