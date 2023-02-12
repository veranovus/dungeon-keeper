use bevy::prelude::*;
use bevy_egui::{egui::{self, RichText, Color32}, EguiContext};

use crate::{
    pawn::prelude::*,
};

pub struct InspectorPlugin;

// NOTE: Size of inspector panel
pub const INSPECTOR_PANEL_SIZE: f32 = 200.0;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, pawn_inspector);
    }
}

fn pawn_inspector(
    mut egui_context: ResMut<EguiContext>,
    query: Query<(&Selectable, &Name, &Health, &Alignment), With<Pawn>>
) {
    egui::SidePanel::left("pawn_inspector")
        .min_width(INSPECTOR_PANEL_SIZE)
        .max_width(INSPECTOR_PANEL_SIZE)
        .exact_width(INSPECTOR_PANEL_SIZE)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.heading("Inspector");
            ui.separator();

            // NOTE: Prepare a list to sort by status of the `Player` component.
            let mut sorted: Vec<(&Selectable, &Name, &Health, &Alignment)> = vec![];

            // NOTE: Display all the selected pawns.
            for tuple in &query {
                sorted.push(tuple);
            }

            // NOTE: Sort and reverse the list
            sorted.sort_by(|a, b| a.3.cmp(&b.3));
            sorted.reverse();

            for (selectable, name, health, alignment) in &sorted {
                if selectable.selected {
                    ui.horizontal(|ui| {
                        let color = alignment.color32();
                        let identifier = alignment.identifier();

                        ui.label(RichText::new(identifier).color(color));

                        ui.separator();

                        ui.label(RichText::new(name.as_str()).strong());
                    });

                    ui.horizontal(|ui| {
                        ui.label("Health:");
                        ui.label(RichText::new(
                            format!("{} | {}", health.maximum, health.current
                        )).color(Color32::GREEN));
                    });
                }
            }
        });
}
