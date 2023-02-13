use bevy::prelude::*;
use bevy_egui::{egui::{self, RichText, Color32}, EguiContext};

use crate::{
    pawn::prelude::*,
    player::resource::prelude::*,
};

pub struct InspectorPlugin;

// NOTE: Size of inspector panel
pub const INSPECTOR_PANEL_SIZE: f32 = 200.0;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, inspector);
    }
}

fn inspector(
    mut egui_context: ResMut<EguiContext>,
    player_resources: Res<PlayerResources>,
    query: Query<(&Selectable, &Name, &Health, &Alignment), With<Pawn>>
) {
    // NOTE: Prepare a list to sort by status of the `Player` component.
    let mut sorted: Vec<(&Selectable, &Name, &Health, &Alignment)> = vec![];

    // NOTE: Display all the selected pawns.
    for tuple in &query {
        sorted.push(tuple);
    }

    // NOTE: Sort and reverse the list
    sorted.sort_by(|a, b| a.3.cmp(&b.3));
    sorted.reverse();

    // NOTE: Render the inspector.
    egui::SidePanel::left("pawn_inspector")
        .min_width(INSPECTOR_PANEL_SIZE)
        .max_width(INSPECTOR_PANEL_SIZE)
        .exact_width(INSPECTOR_PANEL_SIZE)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.heading("Resources");
            ui.separator();

            let mut counter = 0;

            ui.horizontal(|ui| {
                for i in 0..3 {
                    ui.vertical(|ui| {
                        for _ in 0..2 {
                            ui.horizontal(|ui| {
                                let r = &player_resources.resources[counter];

                                ui.label(
                                    RichText::new(
                                        format!("{} : {}", r.material.identifier(), r.quantity).as_str()
                                    ).color(r.material.color32())
                                );
                            });

                            counter += 1;
                        }
                    });

                    if i != 2 {
                        ui.separator();
                    }
                }
            });

            ui.heading("Inspector");
            ui.separator();

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
