//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::{Rotation, One};
use crate::data;
use crate::views;
use crate::views::{DragRotation, ViewMode};
use retain_mut::RetainMut;
use std::cell::RefCell;
use std::rc::Rc;

const MOUSE_WHEEL_ZOOM_FACTOR: f64 = 1.2;

#[derive(Default)]
pub struct GuiState {
    hidpi_factor: f64,
    mouse_drag_origin: [f32; 2]
}

impl GuiState {
    pub fn new(hidpi_factor: f64) -> GuiState {
        GuiState{
            hidpi_factor,
            ..Default::default()
        }
    }
}

fn handle_main_menu(
    ui: &imgui::Ui,
    program_data: &mut data::ProgramData,
    renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
    display: &glium::Display
) {
    let mut orthographic_clicked = false;
    let mut stereographic_clicked = false;
    let mut gnomonic_clicked = false;
    let mut cylindrical_lambert_clicked = false;
    let mut about_clicked = false;
    let mut instructions_clicked = false;

    match ui.begin_main_menu_bar() {
        None => (),
        Some(token) => {
            ui.menu("View", || {
                ui.menu("New", || {
                    if ui.menu_item("Orthographic") {
                        orthographic_clicked = true;
                    }
                    if ui.menu_item("Stereographic") {
                        stereographic_clicked = true;
                    }
                    if ui.menu_item("Gnomonic") {
                        gnomonic_clicked = true;
                    }
                    if ui.menu_item("Lambert cylindrical equal-area") {
                        cylindrical_lambert_clicked = true;
                    }

                });
            });

            ui.menu("Help", || {
                if ui.menu_item("Instructions...") {
                    instructions_clicked = true;
                }
                if ui.menu_item("About...") {
                    about_clicked = true;
                }
            });

            token.end();
        }
    }

    if orthographic_clicked {
        program_data.add_orthographic_view(views::OrthographicView::new(
            program_data, renderer, display
        ));
    }
    if stereographic_clicked {
        program_data.add_stereographic_view(views::StereographicView::new(
            program_data, renderer, display
        ));
    }
    if gnomonic_clicked {
        program_data.add_gnomonic_view(views::GnomonicView::new(
            program_data, renderer, display
        ));
    }
    if cylindrical_lambert_clicked {
        program_data.add_cylindrical_lambert_view(views::CylindricalLambertView::new(
            program_data, renderer, display
        ));
    }

    if instructions_clicked {
        ui.open_popup("Instructions");
        unsafe { imgui::sys::igSetNextWindowSize(
            imgui::sys::ImVec2{ x: 600.0, y: 300.0},
            imgui::sys::ImGuiCond_FirstUseEver as i32
        ); }
    }
    ui.popup_modal("Instructions").build(ui, || {
        ui.text_wrapped("Within a view window, use the left mouse button to change the orientation of the projected globe. \
Use the mouse wheel to zoom in/out.\n\n");
        ui.separator();
        if ui.button("Close") {
            ui.close_current_popup();
        }
    });

    if about_clicked { ui.open_popup("About"); }
    ui.popup_modal("About").build(ui, || {
        ui.text("Map Projections\n
Copyright (c) 2022 Filip Szczerek (ga.software@yahoo.com)\n\n\
This program is licensed under MIT license (see LICENSE.txt for details).\n\n");

        ui.separator();
        if ui.button("Close") {
            ui.close_current_popup();
        }
    });
}

pub fn handle_gui(
    ui: &imgui::Ui,
    gui_state: &mut GuiState,
    program_data: &mut data::ProgramData,
    renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
    display: &glium::Display
) {
    unsafe { imgui::sys::igDockSpaceOverViewport(
        imgui::sys::igGetMainViewport(),
        imgui::sys::ImGuiDockNodeFlags_PassthruCentralNode as i32,
        std::ptr::null()
    ); }

    handle_main_menu(ui, program_data, renderer, display);

    program_data.cylindrical_lambert_views().retain_mut(|view| handle_cylindrical_lambert_view(ui, gui_state, view));
    program_data.gnomonic_views().retain_mut(|view| handle_gnomonic_view(ui, gui_state, view));
    program_data.orthographic_views().retain_mut(|view| handle_orthographic_view(ui, gui_state, view));
    program_data.stereographic_views().retain_mut(|view| handle_stereographic_view(ui, gui_state, view));
}

struct AdjustedImageSize {
    logical_size: [f32; 2],
    physical_size: [u32; 2]
}

/// Adjusts cursor screen position and returns size to be used for an `imgui::Image` (meant to fill the remaining window
/// space) to ensure exact 1:1 pixel rendering when high-DPI scaling is enabled.
fn adjust_pos_for_exact_hidpi_scaling(
    ui: &imgui::Ui,
    vertical_space_after: f32,
    hidpi_factor: f32
) -> AdjustedImageSize {
    let scr_pos = ui.cursor_screen_pos();

    let adjusted_pos_x = if (scr_pos[0] * hidpi_factor).fract() != 0.0 {
        (scr_pos[0] * hidpi_factor).trunc() / hidpi_factor
    } else {
        scr_pos[0]
    };

    let adjusted_pos_y = if (scr_pos[1] * hidpi_factor).fract() != 0.0 {
        (scr_pos[1] * hidpi_factor).trunc() / hidpi_factor
    } else {
        scr_pos[1]
    };

    ui.set_cursor_screen_pos([adjusted_pos_x, adjusted_pos_y]);

    let mut size = ui.content_region_avail();
    size[1] -= vertical_space_after;

    let mut adjusted_size_x = size[0].trunc();
    if (adjusted_size_x * hidpi_factor).fract() != 0.0 {
        adjusted_size_x = (adjusted_size_x * hidpi_factor).trunc() / hidpi_factor;
    }

    let mut adjusted_size_y = size[1].trunc();
    if (adjusted_size_y * hidpi_factor).fract() != 0.0 {
        adjusted_size_y = (adjusted_size_y * hidpi_factor).trunc() / hidpi_factor;
    }

    let physical_size = [
        (adjusted_size_x * hidpi_factor).trunc() as u32,
        (adjusted_size_y * hidpi_factor).trunc() as u32
    ];

    AdjustedImageSize{
        logical_size: [adjusted_size_x, adjusted_size_y],
        physical_size
    }
}

fn handle_view_common(ui: &imgui::Ui, gui_state: &mut GuiState, view: &mut views::ViewBase) {
    ui.button("reset");
    if ui.is_item_active() {
        view.set_orientation(cgmath::Basis3::one());
    }
    if ui.is_item_hovered() {
        ui.tooltip_text("Reset view to default orientation");
    }
    ui.same_line();

    unsafe { imgui::sys::igSeparatorEx(imgui::sys::ImGuiSeparatorFlags_Vertical as i32); }
    ui.same_line();

    if ui.checkbox("graticule", &mut view.draw_graticule) {
        view.refresh();
    }
    ui.same_line();

    unsafe { imgui::sys::igSeparatorEx(imgui::sys::ImGuiSeparatorFlags_Vertical as i32); }
    ui.same_line();

    if ui.radio_button_bool("texture##2", view.view_mode() == ViewMode::GlobeTexture) {
        view.set_view_mode(ViewMode::GlobeTexture);
    }
    ui.same_line();
    if ui.radio_button_bool("lines##2", view.view_mode() == ViewMode::VectorMap) {
        view.set_view_mode(ViewMode::VectorMap);
    }
    ui.same_line();

    unsafe { imgui::sys::igSeparatorEx(imgui::sys::ImGuiSeparatorFlags_Vertical as i32); }
    ui.same_line();

    ui.text("rotation:");
    ui.same_line();
    if ui.radio_button_bool("NSEW##1", view.drag_rotation() == DragRotation::NSEW) {
        view.set_drag_rotation(DragRotation::NSEW);
    }
    ui.same_line();
    if ui.radio_button_bool("free##1", view.drag_rotation() == DragRotation::Free) {
        view.set_drag_rotation(DragRotation::Free);
    }

    let hidpi_f = gui_state.hidpi_factor as f32;

    let adjusted = adjust_pos_for_exact_hidpi_scaling(ui, 0.0, hidpi_f);

    view.update_size(
        adjusted.physical_size[0],
        adjusted.physical_size[1]
    );

    let img_pos_in_app_window = ui.cursor_screen_pos();

    let image_start_pos = ui.cursor_pos();
    imgui::Image::new(view.draw_buf_id(), adjusted.logical_size).build(ui);

    let mouse_pos_in_app_window = ui.io().mouse_pos;
    if ui.is_item_clicked_with_button(imgui::MouseButton::Left) {
        gui_state.mouse_drag_origin = [
            mouse_pos_in_app_window[0] - img_pos_in_app_window[0],
            mouse_pos_in_app_window[1] - img_pos_in_app_window[1]
        ];
    }
    if ui.is_item_hovered() {
        let wheel = ui.io().mouse_wheel;
        if wheel != 0.0 {
            let zoom_factor = MOUSE_WHEEL_ZOOM_FACTOR.powf(wheel as f64);
            view.zoom_by(zoom_factor);
        }

        if ui.is_mouse_dragging(imgui::MouseButton::Left) {
            let delta = ui.mouse_drag_delta_with_button(imgui::MouseButton::Left);
            if delta[0] != 0.0 || delta[1] != 0.0 {
                let drag_start: [f32; 2] = [
                    -1.0 + 2.0 * (gui_state.mouse_drag_origin[0] / adjusted.logical_size[0]),
                    -(-1.0 + 2.0 * (gui_state.mouse_drag_origin[1] / adjusted.logical_size[1]))
                ];

                let drag_end = [
                    drag_start[0] + 2.0 * delta[0] / adjusted.logical_size[0],
                    drag_start[1] - 2.0 * delta[1] / adjusted.logical_size[1]
                ];

                view.rotate_by_dragging(drag_start, drag_end);
            }
            ui.reset_mouse_drag_delta(imgui::MouseButton::Left);
            gui_state.mouse_drag_origin = [
                mouse_pos_in_app_window[0] - img_pos_in_app_window[0],
                mouse_pos_in_app_window[1] - img_pos_in_app_window[1]
            ];
        }
    }

    ui.set_cursor_pos(image_start_pos);
    let _disabled = ui.begin_disabled(true);
    let _token1 = ui.push_style_color(imgui::StyleColor::Text, [0.0, 0.0, 0.0, 1.0]);
    let _token2 = ui.push_style_color(imgui::StyleColor::Button, [1.0, 1.0, 1.0, 0.8]);

    let dir_of_lonlat00 = cgmath::Vector3{ x: 1.0, y: 0.0, z: 0.0 };
    let reor = view.orientation().invert().as_ref() * dir_of_lonlat00;

    let central_longitude = if reor.x > 0.0 {
        cgmath::Deg::from(cgmath::Rad((reor.y / reor.z.asin().cos()).asin())).0
    } else {
        cgmath::Deg::from(cgmath::Rad(reor.y.signum() * std::f64::consts::PI - (reor.y / reor.z.asin().cos()).asin())).0
    };
    let central_latitude = cgmath::Deg::from(cgmath::Rad(reor.z.asin())).0;

    let lon_str = format!("{:.1}° {}", central_longitude.abs(), if central_longitude >= 0.0 { "E" } else { "W" });
    let lat_str = format!("{:.1}° {}", central_latitude.abs(), if central_latitude >= 0.0 { "N" } else { "S" });
    ui.small_button(&format!("{} {}", lon_str, lat_str));
}

/// Returns `false` if view should be deleted.
fn handle_cylindrical_lambert_view(
    ui: &imgui::Ui,
    gui_state: &mut GuiState,
    view: &mut views::CylindricalLambertView
) -> bool {
    let mut opened = true;

    imgui::Window::new(ui, &format!("Lambert cylindrical###cylindrical_lambert_{}", view.unique_id()))
        .size([640.0, 320.0], imgui::Condition::FirstUseEver)
        .opened(&mut opened)
        .build(|| {
            handle_view_common(ui, gui_state, view.base_mut());
        }
    );

    opened
}

/// Returns `false` if view should be deleted.
fn handle_gnomonic_view(
    ui: &imgui::Ui,
    gui_state: &mut GuiState,
    view: &mut views::GnomonicView
) -> bool {
    let mut opened = true;

    imgui::Window::new(ui, &format!("Gnomonic###gnomonic_{}", view.unique_id()))
        .size([640.0, 640.0], imgui::Condition::FirstUseEver)
        .opened(&mut opened)
        .build(|| {
            handle_view_common(ui, gui_state, view.base_mut());
        }
    );

    opened
}

/// Returns `false` if view should be deleted.
fn handle_orthographic_view(
    ui: &imgui::Ui,
    gui_state: &mut GuiState,
    view: &mut views::OrthographicView
) -> bool {
    let mut opened = true;

    imgui::Window::new(ui, &format!("Orthographic###orthographic_{}", view.unique_id()))
        .size([640.0, 640.0], imgui::Condition::FirstUseEver)
        .opened(&mut opened)
        .build(|| {
            handle_view_common(ui, gui_state, view.base_mut());
        }
    );

    opened
}

/// Returns `false` if view should be deleted.
fn handle_stereographic_view(
    ui: &imgui::Ui,
    gui_state: &mut GuiState,
    view: &mut views::StereographicView
) -> bool {
    let mut opened = true;

    imgui::Window::new(ui, &format!("Stereographic###stereographic_{}", view.unique_id()))
        .size([640.0, 640.0], imgui::Condition::FirstUseEver)
        .opened(&mut opened)
        .build(|| {
            handle_view_common(ui, gui_state, view.base_mut());
        }
    );

    opened
}
