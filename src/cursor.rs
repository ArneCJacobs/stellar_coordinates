use crate::{Input, KeyCode, MouseButton, Res};
use smooth_bevy_cameras::controllers::fps::FpsCameraController;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;

// hides mouse
pub fn cursor_grab_system(
    // mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut camera_controller_query: Query<&mut FpsCameraController>,
    egui_context_opt: Option<EguiContexts>, // egui context added by bevy_inspector_egui
    egui_context2_opt: Option<bevy_egui::EguiContexts>, // egui context added by bevy_egui
) {
    let window = windows.get_primary_mut().unwrap();

    let mut camera_controller = camera_controller_query.get_single_mut().unwrap();

    let hovering_over_egui = match egui_context_opt {
        Some(mut egui_context) => egui_context.ctx_mut().is_pointer_over_area(),
        None => match egui_context2_opt {
            Some(mut egui_context) => egui_context.ctx_mut().is_pointer_over_area(),
            None => false
        }
    };

    if btn.just_pressed(MouseButton::Left) && !hovering_over_egui {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
        camera_controller.enabled = true;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
        camera_controller.enabled = false;
    }
}
