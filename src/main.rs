//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

#[macro_use]
extern crate imgui_glium_renderer;

mod data;
mod draw_buffer;
mod gui;
mod runner;
mod views;

use std::{rc::Rc, io::Write};

fn main() {
    let runner = runner::create_runner(18.0);

    let mut data = data::ProgramData::new(runner.display());

    let mut gui_state = gui::GuiState::new(runner.platform().hidpi_factor());

    runner.main_loop(move |_, ui, display, renderer| {
        gui::handle_gui(ui, &mut gui_state, &mut data, renderer, display);
    });
}
