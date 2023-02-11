use bevy::prelude::*;
use log::error;

use super::prelude::*;

pub mod prelude {
    pub use super::{
        Health,
    };
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, setup_health);
    }
}

// NOTE: Base health for every creature for now
// TODO: Change this later on.
pub const BASE_HEALTH: u32 = 5;

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

// NOTE: Contains the maximum and current health of the pawn,
//       this component must be added after `PawnStats`.
#[derive(Component)]
pub struct Health {
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

        let test = crate::util::dice::roll(3, crate::util::dice::Die::D12(0), 3, crate::util::dice::Advantage::Normal);
        println!("Dice Roll: {:?}", test);

        health.maximum = BASE_HEALTH as i32 + get_stat_bonus(vitality);
        health.current = health.maximum;
    }
}