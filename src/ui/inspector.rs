use bevy::prelude::*;
use bevy_egui::{egui::{self, RichText, Color32}, EguiContext};

use crate::{
    player::component::prelude::*,
    pawn::prelude::*,
};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, pawn_inspector);
    }
}

fn pawn_inspector(
    mut egui_context: ResMut<EguiContext>,
    query: Query<(&Selectable, &Name, &Health, Option<&Player>), With<Pawn>>
) {
    egui::SidePanel::left("pawn_inspector")
        .min_width(200.0)
        .max_width(200.0)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.heading("Inspector");
            ui.separator();

            // NOTE: Display all the selected pawns.
            for (selectable, name, health, player) in &query {
                if selectable.selected {
                    ui.horizontal(|ui| {
                        let color: Color32;
                        let identifier;

                        if player.is_some() {
                            identifier = "P";
                            color = Color32::from_rgb(25, 25, 255);
                        } else {
                            identifier = "E";
                            color = Color32::from_rgb(255, 25, 25);
                        }

                        ui.label(RichText::new(identifier).color(color));

                        ui.separator();

                        ui.label(RichText::new(name.as_str()).strong());
                    });

                    ui.label(
                        format!("Health: {}/{}", health.maximum, health.current)
                    );
                }
            }
        });
}