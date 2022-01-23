//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use cgmath::One;
use std::rc::Rc;
use crate::data;
use crate::views::{base::ViewBase};
use std::cell::RefCell;

pub struct OrthographicView {
    base: ViewBase,
}

impl OrthographicView {
    pub fn new(
        program_data: &data::ProgramData,
        renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
        display: &glium::Display
    ) -> OrthographicView {
        OrthographicView{
            base: ViewBase::new(
                OrthographicView::initial_orientation(),
                program_data,
                Rc::clone(&program_data.gl_programs.orthographic.lines),
                Rc::clone(&program_data.gl_programs.orthographic.triangles),
                display,
                renderer
            ),
        }
    }

    pub fn unique_id(&self) -> u32 { self.base.unique_id() }

    pub fn base_mut(&mut self) -> &mut ViewBase { &mut self.base }

    /// Returns identity matrix: observer facing long. 0°, lat. °.
    fn initial_orientation() -> cgmath::Basis3<f64> {
        cgmath::Basis3::one()
    }
}
