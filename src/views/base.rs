//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use crate::draw_buffer::{Sampling, DrawBuffer};
use crate::data::{LonLatGlBuffers, ProgramData, ToArray};
use cgmath::{Basis3, Vector3, InnerSpace, Rotation3, One, Matrix3};
use std::rc::Rc;
use std::cell::RefCell;
use glium::Surface;

#[derive(Copy, Clone, PartialEq)]
pub enum DragRotation {
    NSEW,
    Free
}

#[derive(Copy, Clone, PartialEq)]
pub enum ViewMode {
    VectorMap,
    GlobeTexture
}

mod uniform_names {
    pub const UNIFORM_COLOR: &str = "uniform_color";
}

/// Base struct representing a view.
///
/// The underlying globe being projected is oriented as per `orientation`. The globe is centered
/// at (0, 0, 0) with radius equal to 1.
///
/// The camera is located at (1, 0, 0) looking at (0, 0, 0); when `orientation`
/// is an identity matrix, the observer looks at lat. 0°, long. 0° (i.e. the line
/// from globe center to lat. 0°, long. 0° overlaps with the X axis).
///
pub struct ViewBase {
    unique_id: u32,

    orientation: Basis3<f64>,

    pub draw_graticule: bool,

    wh_ratio: f32,

    angle_ns: cgmath::Rad<f64>,

    angle_ew: cgmath::Rad<f64>,

    view_mode: ViewMode,

    zoom: f64,

    drag_rotation: DragRotation,

    draw_buf: DrawBuffer,

    globe_gl_buf: LonLatGlBuffers,

    graticule_gl_buf: LonLatGlBuffers,

    map_gl_buf: LonLatGlBuffers,

    globe_texture: Rc<glium::texture::texture2d::Texture2d>,

    lines_gl_prog: Rc<glium::Program>,

    tris_gl_prog: Rc<glium::Program>
}

impl ViewBase {
    pub fn refresh(&self) {
        self.render();
    }

    pub fn view_mode(&self) -> ViewMode { self.view_mode }

    pub fn set_view_mode(&mut self, view_mode: ViewMode) {
        self.view_mode = view_mode;
        self.render();
    }

    pub fn draw_buf_id(&self) -> imgui::TextureId { self.draw_buf.id() }

    pub fn drag_rotation(&self) -> DragRotation { self.drag_rotation }

    pub fn zoom_by(&mut self, relative_zoom: f64) {
        self.zoom *= relative_zoom;
        if self.zoom < 0.5 { self.zoom = 0.5; }
        self.render();
    }

    pub fn orientation(&self) -> &cgmath::Basis3<f64> { &self.orientation }

    pub fn set_orientation(&mut self, orientation: cgmath::Basis3<f64>) {
        if orientation != Basis3::one() {
            self.drag_rotation = DragRotation::Free;
        };
        self.orientation = orientation;
        // TODO: if rotation mode is NSEW, we should actually calculate current angles here
        self.angle_ew = cgmath::Rad(0.0);
        self.angle_ns = cgmath::Rad(0.0);
        self.render();
    }

    pub fn set_drag_rotation(&mut self, drag_rotation: DragRotation) {
        self.drag_rotation = drag_rotation;
        if drag_rotation == DragRotation::NSEW {
            self.orientation = Basis3::one();
            self.angle_ew = cgmath::Rad(0.0);
            self.angle_ns = cgmath::Rad(0.0);
            self.render();
        }
    }

    pub(in crate::views) fn render(
        &self,
        //view_specific_uniforms: glium::uniforms::UniformsStorage<'_, T, R>
    ) {
        // no need for a depth test; depending on particular view, either the projection clips the rear hemisphere,
        // or the vertex shader outputs vertices on a plane
        let draw_params = glium::DrawParameters::default();

        let uniforms = uniform! {
            globe_orientation: Matrix3::from(self.orientation).cast::<f32>().unwrap().to_array(),
            zoom: self.zoom as f32,
            wh_ratio : self.wh_ratio,
            source_texture: glium::uniforms::Sampler::new(&*self.globe_texture)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
        };

        let mut target = self.draw_buf.frame_buf();

        match self.view_mode {
            ViewMode::GlobeTexture => {
                target.clear_color(0.5, 0.5, 0.5, 1.0);

                target.draw(
                    &*self.globe_gl_buf.vertices,
                    &*self.globe_gl_buf.indices,
                    &*self.tris_gl_prog,
                    &uniforms,
                    &draw_params
                ).unwrap();
            },

            ViewMode::VectorMap => {
                target.clear_color(0.87, 0.87, 0.87, 1.0);

                let uniforms = uniforms.clone().add(uniform_names::UNIFORM_COLOR, [0f32, 0f32, 0f32, 1f32]);
                target.draw(
                    &*self.map_gl_buf.vertices,
                    &*self.map_gl_buf.indices,
                    &self.lines_gl_prog,
                    &uniforms,
                    &draw_params
                ).unwrap();
            }
        }

        if self.draw_graticule {
            let uniforms = uniforms.add(uniform_names::UNIFORM_COLOR, [0.6f32, 0.6f32, 0.6f32, 1f32]);
            target.draw(
                &*self.graticule_gl_buf.vertices,
                &*self.graticule_gl_buf.indices,
                &self.lines_gl_prog,
                &uniforms,
                &draw_params
            ).unwrap();
        }

        self.draw_buf.update_storage_buf();
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        if height == 0 { return; }

        if self.draw_buf.update_size(width, height) {
            self.wh_ratio = width as f32 / height as f32;
            self.render()
        }
    }

    pub (in crate::views) fn unique_id(&self) -> u32 { self.unique_id }

    pub(in crate::views) fn new(
        orientation: Basis3<f64>,
        program_data: &ProgramData,
        lines_gl_prog: Rc<glium::Program>,
        tris_gl_prog: Rc<glium::Program>,
        display: &glium::Display,
        renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>
    ) -> ViewBase {
        let drag_rotation = if orientation != Basis3::one() {
            DragRotation::Free
        } else {
            DragRotation::NSEW
        };

        ViewBase{
            unique_id: program_data.new_unique_id(),
            orientation,
            draw_graticule: true,
            wh_ratio: 1.0,
            view_mode: ViewMode::GlobeTexture,
            angle_ns: cgmath::Rad(0.0),
            angle_ew: cgmath::Rad(0.0),
            drag_rotation,
            zoom: 1.0,
            draw_buf: DrawBuffer::new(
                Sampling::Multi,
                &program_data.gl_programs.texture_copy_single,
                &program_data.gl_programs.texture_copy_multi,
                &program_data.unit_quad,
                display,
                &renderer
            ),
            globe_gl_buf: program_data.globe_gl_buf.clone(),
            graticule_gl_buf: program_data.graticule_gl_buf.clone(),
            map_gl_buf: program_data.map_gl_buf.clone(),
            globe_texture: program_data.globe_texture.clone(),
            lines_gl_prog,
            tris_gl_prog
        }
    }

    /// Elements of `start` and `end` denote normalized mouse position within the view,
    /// with values from [-1, 1] (i.e., bottom-left is [-1, -1], and top-right is [1, 1]).
    pub fn rotate_by_dragging(&mut self, start: [f32; 2], end: [f32; 2]) {
        match self.drag_rotation {
            // simulates "space ball" rotation
            DragRotation::Free => {
                let start_vec = Vector3{ x: 1.0, y: start[0] as f64, z: start[1] as f64 };
                let end_vec = Vector3{ x: 1.0, y: end[0] as f64, z: end[1] as f64 };

                let axis_of_rotation = start_vec.cross(end_vec).normalize();
                let angle = cgmath::Rad(
                    1.0 / self.zoom * ((start[0] - end[0]).powi(2) + (start[1] - end[1]).powi(2)).sqrt() as f64
                );

                let rotation = cgmath::Basis3::from_axis_angle(axis_of_rotation, angle);

                self.orientation = rotation * self.orientation;
            },

            DragRotation::NSEW => {
                let new_angle_ns = self.angle_ns + cgmath::Rad(1.0 / self.zoom * (start[1] - end[1]) as f64);
                if new_angle_ns.0.abs() <= cgmath::Rad::from(cgmath::Deg(90.0)).0 {
                    self.angle_ns = new_angle_ns;
                }

                self.angle_ew += cgmath::Rad(1.0 / self.zoom * (end[0] - start[0]) as f64);

                let rotation_ns = cgmath::Basis3::from_angle_y(self.angle_ns);
                let rotation_ew = cgmath::Basis3::from_angle_z(self.angle_ew);

                self.orientation = rotation_ns * rotation_ew;
            }
        }

        self.render();
    }
}
