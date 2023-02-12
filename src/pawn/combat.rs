use bevy::prelude::*;
use log::error;

use super::prelude::*;

pub mod prelude {
    pub use super::{
        HitDie,
        Health,
        DamageType,
        Resistance,
    };
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, setup_health);
    }
}

#[allow(dead_code)]
// NOTE: All possible damage types, as a rule
//       no pawn should be resistant to force damage.
pub enum DamageType {
    Invalid,
    Slashing,
    Fire,
    Cold,
    Acid,
    Thunder,
    Lightning,
    Radiant,
    Necrotic,
    Force,
}

#[derive(Component)]
// NOTE: Contains the damage resistances and immunities of an entity.
pub struct Resistance {
    pub resistances: Vec<DamageType>,
    pub immunities: Vec<DamageType>,
}

#[allow(dead_code)]
// NOTE: Die that is used to determine a pawn's health.
pub enum HitDie {
    D4,
    D6,
    D8,
    D10,
    D12,
}

impl HitDie {
    pub fn value(&self) -> u32 {
        match self {
            HitDie::D4 => 4,
            HitDie::D6 => 6,
            HitDie::D8 => 8,
            HitDie::D10 => 10,
            HitDie::D12 => 12,
        }
    }
}

// NOTE: Contains the maximum and current health of the pawn,
//       this component must be added after `PawnStats`.
#[derive(Component)]
pub struct Health {
    pub hit_die: HitDie,
    pub current: i32,
    pub maximum: i32,
}

// NOTE: Sets up the health component when its first added to a pawn,
//       also checks if that pawn has `PawnStats` component which is required for `Health`.
fn setup_health(mut query: Query<(&mut Health, Option<&PawnStats>), (With<Pawn>, Added<Health>)>) {
    for (mut health, stats) in &mut query {
        let vitality = if let Some(s) = stats {
            s.vitality
        } else {
            error!("Pawn is missing `PawnStats` but has a dependant trait `Health`.");
            panic!();
        };

        health.maximum = health.hit_die.value() as i32 + get_stat_bonus(vitality);
        health.current = health.maximum;
    }
}