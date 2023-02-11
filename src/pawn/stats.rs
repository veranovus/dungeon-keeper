use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        PawnStats,
        DEFAULT_PAWN_STAT,
        get_stat_bonus,
    };
}

// NOTE: Default value for every stat.
pub const DEFAULT_PAWN_STAT: u32 = 8;

// NOTE: Collection of base statistics for any given pawn,
//       the default value of every stat is `DEFAULT_PAWN_STAT`.
//       - Vitality     -> General durability, and health of a pawn.
//       - Strenght     -> Strength of a pawn, used to calculate melee damage.
//       - Dexterity    -> Determines a pawns, ranged damage and dodge stats.
//       - Intelligence -> Determines the wizard spell damage, and effectivenes.
//       - Wisdom       -> Determines the cleric spell damage, and effectivenes.
#[derive(Component)]
pub struct PawnStats {
    pub vitality: u32,
    pub strenght: u32,
    pub dexterity: u32,
    pub intelligence: u32,
    pub wisdom: u32,
}

impl Default for PawnStats {
    fn default() -> Self {
        return Self {
            vitality: DEFAULT_PAWN_STAT,
            strenght: DEFAULT_PAWN_STAT,
            dexterity: DEFAULT_PAWN_STAT,
            intelligence: DEFAULT_PAWN_STAT,
            wisdom: DEFAULT_PAWN_STAT,
        };
    }
}

// NOTE: Returns the stat bonus for any given stat.
pub fn get_stat_bonus(value: u32) -> i32 {
    return value as i32 - DEFAULT_PAWN_STAT as i32;
}