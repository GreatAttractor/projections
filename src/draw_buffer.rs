//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use glium::Surface;
use glium::texture::{
    depth_texture2d_multisample::DepthTexture2dMultisample,
    depth_texture2d::DepthTexture2d,
    texture2d_multisample::Texture2dMultisample,
    texture2d::Texture2d,
};
use std::cell::RefCell;
use std::rc::Rc;

const INITIAL_DRAW_BUF_SIZE: u32 = 256;

const COLOR_FORMAT: glium::texture::UncompressedFloatFormat = glium::texture::UncompressedFloatFormat::U8U8U8U8;

const DEPTH_FORMAT: glium::texture::DepthFormat = glium::texture::DepthFormat::I24;

const NUM_SAMPLES: u32 = 8;

#[derive(Copy, Clone, PartialEq)]
pub enum Sampling { Single, Multi }

/// Contains (draw buffer, depth buffer).
enum Buffers {
    SingleSampling(Texture2d, DepthTexture2d),
    MultiSampling(Texture2dMultisample, DepthTexture2dMultisample)
}

impl Buffers {
    fn sampling(&self) -> Sampling {
        match self {
            Buffers::SingleSampling(_, _) => Sampling::Single,
            Buffers::MultiSampling(_, _) => Sampling::Multi
        }
    }
}

/// Draw buffer for double-buffered views.
pub struct DrawBuffer {
    id: imgui::TextureId,

    renderer: Rc<RefCell<imgui_glium_renderer::Renderer>>,

    display: glium::Display,

    /// Used for rendering.
    draw_bufs: Buffers,

    /// Used for storage and displaying.
    storage_buf: Rc<Texture2d>,

    /// GL program to handle texture copying with single-sampling.
    texture_copy_single_gl_prog: Rc<glium::Program>,

    /// GL program to handle texture copying with multi-sampling.
    texture_copy_multi_gl_prog: Rc<glium::Program>,

    unit_quad: Rc<glium::VertexBuffer<crate::data::XyVertex>>
}

impl DrawBuffer {
    pub fn set_sampling(&mut self, sampling: Sampling) {
        let (id, draw_bufs, storage_buf) = DrawBuffer::create(
            sampling,
            &Some(self.id),
            self.width(),
            self.height(),
            COLOR_FORMAT,
            &self.display,
            &mut self.renderer.borrow_mut()
        );
        self.id = id;
        self.draw_bufs = draw_bufs;
        self.storage_buf = storage_buf;
    }

    /// If something was rendered using the result of `frame_buf()`, this method must be called afterwards.
    pub fn update_storage_buf(&self) {
        let mut fbo = glium::framebuffer::SimpleFrameBuffer::new(&self.display, &*self.storage_buf).unwrap();

        match &self.draw_bufs {
            Buffers::SingleSampling(draw_buf, _) => {
                let uniforms = uniform! {
                    source_texture: draw_buf.sampled()
                };

                fbo.draw(
                    &*self.unit_quad,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan),
                    &self.texture_copy_single_gl_prog,
                    &uniforms,
                    &Default::default()
                ).unwrap();
            },

            Buffers::MultiSampling(draw_buf, _) => {
                let uniforms = uniform! {
                    source_texture: draw_buf.sampled()
                };

                fbo.draw(
                    &*self.unit_quad,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan),
                    &self.texture_copy_multi_gl_prog,
                    &uniforms,
                    &Default::default()
                ).unwrap();
            },
        };
    }

    pub fn storage_buf(&self) -> &Rc<Texture2d> {
        &self.storage_buf
    }

    pub fn frame_buf(&self) -> glium::framebuffer::SimpleFrameBuffer {
        match &self.draw_bufs {
            Buffers::SingleSampling(draw_buf, depth_buf) => glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                &self.display, draw_buf, depth_buf
            ).unwrap(),

            Buffers::MultiSampling(draw_buf, depth_buf) => glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                &self.display, draw_buf, depth_buf
            ).unwrap()
        }
    }

    pub fn width(&self) -> u32 { self.storage_buf.width() }

    pub fn height(&self) -> u32 { self.storage_buf.height() }

    pub fn new(
        sampling: Sampling,
        texture_copy_single_gl_prog: &Rc<glium::Program>,
        texture_copy_multi_gl_prog: &Rc<glium::Program>,
        unit_quad: &Rc<glium::VertexBuffer<crate::data::XyVertex>>,
        display: &glium::Display,
        renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>
    ) -> DrawBuffer {
        let (id, draw_bufs, storage_buf) = DrawBuffer::create(
            sampling,
            &None,
            INITIAL_DRAW_BUF_SIZE,
            INITIAL_DRAW_BUF_SIZE,
            COLOR_FORMAT,
            display,
            &mut renderer.borrow_mut()
        );

        DrawBuffer {
            id,
            display: display.clone(),
            renderer: Rc::clone(renderer),
            draw_bufs,
            storage_buf,
            unit_quad: Rc::clone(unit_quad),
            texture_copy_single_gl_prog: Rc::clone(texture_copy_single_gl_prog),
            texture_copy_multi_gl_prog: Rc::clone(texture_copy_multi_gl_prog)
        }
    }

    pub fn new_with_size(
        sampling: Sampling,
        texture_copy_single_gl_prog: &Rc<glium::Program>,
        texture_copy_multi_gl_prog: &Rc<glium::Program>,
        unit_quad: &Rc<glium::VertexBuffer<crate::data::XyVertex>>,
        display: &glium::Display,
        renderer: &Rc<RefCell<imgui_glium_renderer::Renderer>>,
        width: u32,
        height: u32
    ) -> DrawBuffer {
        let (id, draw_bufs, storage_buf) = DrawBuffer::create(
            sampling,
            &None,
            width,
            height,
            COLOR_FORMAT,
            display,
            &mut renderer.borrow_mut()
        );

        DrawBuffer {
            id,
            display: display.clone(),
            renderer: Rc::clone(renderer),
            draw_bufs,
            storage_buf,
            unit_quad: Rc::clone(unit_quad),
            texture_copy_single_gl_prog: Rc::clone(texture_copy_single_gl_prog),
            texture_copy_multi_gl_prog: Rc::clone(texture_copy_multi_gl_prog)
        }
    }

    pub fn id(&self) -> imgui::TextureId {
        self.id
    }

    fn create(
        sampling: Sampling,
        prev_id: &Option<imgui::TextureId>,
        width: u32,
        height: u32,
        format: glium::texture::UncompressedFloatFormat,
        display: &glium::Display,
        renderer: &mut imgui_glium_renderer::Renderer
    ) -> (imgui::TextureId, Buffers, Rc<Texture2d>) {
        let draw_bufs = match sampling {
            Sampling::Single => Buffers::SingleSampling(
                Texture2d::empty_with_format(
                    display,
                    format,
                    glium::texture::MipmapsOption::NoMipmap,
                    width,
                    height
                ).unwrap(),
                DepthTexture2d::empty_with_format(
                    display,
                    DEPTH_FORMAT,
                    glium::texture::MipmapsOption::NoMipmap,
                    width,
                    height
                ).unwrap()
            ),

            Sampling::Multi => Buffers::MultiSampling(
                Texture2dMultisample::empty_with_format(
                    display,
                    format,
                    glium::texture::MipmapsOption::NoMipmap,
                    width,
                    height,
                    NUM_SAMPLES
                ).unwrap(),
                DepthTexture2dMultisample::empty_with_format(
                    display,
                    DEPTH_FORMAT,
                    glium::texture::MipmapsOption::NoMipmap,
                    width,
                    height,
                    NUM_SAMPLES
                ).unwrap()
            )
        };

        let storage_buf = std::rc::Rc::new(Texture2d::empty_with_format(
            display,
            // no alpha here, otherwise it would leak the background when fed to Dear ImGUI's `Image` widget
            glium::texture::UncompressedFloatFormat::U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            width,
            height
        ).unwrap());

        let imgui_tex = imgui_glium_renderer::Texture {
            texture: storage_buf.clone(),
            sampler: glium::uniforms::SamplerBehavior {
                magnify_filter: glium::uniforms::MagnifySamplerFilter::Linear,
                minify_filter: glium::uniforms::MinifySamplerFilter::Linear,
                ..Default::default()
            },
        };

        let id = match prev_id {
            None => renderer.textures().insert(imgui_tex),
            Some(prev_id) => {
                renderer.textures().replace(*prev_id, imgui_tex);
                *prev_id
            }
        };

        (id, draw_bufs, storage_buf)
    }

    /// If size changes, underlying texture is created anew.
    pub fn update_size(
        &mut self,
        width: u32,
        height: u32
    ) -> bool {
        if width != self.storage_buf.width() || height != self.storage_buf.height() {
            let (id, draw_bufs, storage_buf) = DrawBuffer::create(
                self.draw_bufs.sampling(),
                &Some(self.id),
                width,
                height,
                COLOR_FORMAT,
                &self.display,
                &mut self.renderer.borrow_mut()
            );
            self.id = id;
            self.draw_bufs = draw_bufs;
            self.storage_buf = storage_buf;

            true
        } else {
            false
        }
    }
}
