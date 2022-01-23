//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use crate::views::{
    CylindricalLambertView,
    GnomonicView,
    OrthographicView,
    StereographicView
};
use glium::CapabilitiesSource;
use image::{GenericImageView};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct LonLatVertex {
    // values in degrees; -180° ⩽ longitude ⩽ 180°, -90° ⩽ latitude ⩽ 90°
    lonlat_position: [f32; 2]
}
glium::implement_vertex!(LonLatVertex, lonlat_position);

#[derive(Copy, Clone)]
pub struct XyVertex {
    position: [f32; 2]
}
glium::implement_vertex!(XyVertex, position);

#[derive(Copy, Clone)]
pub struct XyzVertex {
    position: [f32; 3]
}
glium::implement_vertex!(XyzVertex, position);

pub trait ToArray {
    type Output;
    fn to_array(&self) -> Self::Output;
}

impl<T: Copy> ToArray for cgmath::Matrix3<T>
{
    type Output = [[T; 3]; 3];
    fn to_array(&self) -> Self::Output {
        (*self).into()
    }
}

impl<T: Copy> ToArray for cgmath::Matrix4<T>
{
    type Output = [[T; 4]; 4];
    fn to_array(&self) -> Self::Output {
        (*self).into()
    }
}

pub struct GlProgramPair {
    /// OpenGL program for rendering lines.
    pub lines: Rc<glium::Program>,
    /// OpenGL program for rendering triangles.
    pub triangles: Rc<glium::Program>
}

pub struct OpenGlPrograms {
    pub cylindrical_lambert: GlProgramPair,
    pub gnomonic: GlProgramPair,
    pub orthographic: GlProgramPair,
    pub stereographic: GlProgramPair,
    pub texture_copy_single: Rc<glium::Program>,
    pub texture_copy_multi: Rc<glium::Program>
}

#[derive(Clone)]
pub struct LonLatGlBuffers {
    pub vertices: Rc<glium::VertexBuffer<LonLatVertex>>,
    pub indices: Rc<glium::IndexBuffer<u32>>,
}

pub struct ProgramData {
    id_counter: Rc<RefCell<u32>>,

    pub gl_programs: OpenGlPrograms,

    pub unit_quad: Rc<glium::VertexBuffer<XyVertex>>,

    pub globe_texture: Rc<glium::Texture2d>,

    pub globe_gl_buf: LonLatGlBuffers,

    pub graticule_gl_buf: LonLatGlBuffers,

    pub map_gl_buf: LonLatGlBuffers,

    pub cylindrical_lambert_views: Vec<CylindricalLambertView>,

    pub gnomonic_views: Vec<GnomonicView>,

    pub orthographic_views: Vec<OrthographicView>,

    pub stereographic_views: Vec<StereographicView>
}

fn create_gl_program_pair(vertex_shader_source: &str, display: &glium::Display) -> GlProgramPair {
    GlProgramPair{
        lines: Rc::new(program!(display,
            330 => {
                vertex: vertex_shader_source,
                geometry: include_str!("resources/shaders/lines.geom"),
                fragment: include_str!("resources/shaders/uniform_color.frag")
            }
        ).unwrap()),

        triangles: Rc::new(program!(display,
                330 => {
                    vertex: vertex_shader_source,
                    geometry: include_str!("resources/shaders/tris.geom"),
                    fragment: include_str!("resources/shaders/globe_texturing.frag")
                }
        ).unwrap())
    }
}

impl ProgramData {
    pub fn new(display: &glium::Display) -> ProgramData {
        let globe_texture = Rc::new(create_texture_from_image(
            "data/world.topo.bathy.200412.3x8192x4096.jpg",
            display
        ));

        let globe_gl_buf = create_globe_mesh(cgmath::Deg(2.0), display);

        let graticule_gl_buf = create_graticule(cgmath::Deg(10.0), 10, display);

        let map_gl_buf = create_map_from_shape_file(
            "data/ne_10m_coastline/ne_10m_coastline.shp",
            display
        );

        let texture_copy_single = Rc::new(program!(display,
            330 => {
                vertex: include_str!("resources/shaders/pass-through.vert"),
                fragment: include_str!("resources/shaders/texturing.frag"),
            }
        ).unwrap());

        let texture_copy_multi = Rc::new(program!(display,
            330 => {
                vertex: include_str!("resources/shaders/pass-through.vert"),
                fragment: include_str!("resources/shaders/texturing_multi-sample.frag"),
            }
        ).unwrap());

        let cylindrical_lambert = create_gl_program_pair(
            include_str!("resources/shaders/cylindrical_lambert.vert"),
            display
        );
        let gnomonic = create_gl_program_pair(
            include_str!("resources/shaders/gnomonic.vert"),
            display
        );
        let orthographic = create_gl_program_pair(
            include_str!("resources/shaders/orthographic.vert"),
            display
        );
        let stereographic = create_gl_program_pair(
            include_str!("resources/shaders/stereographic.vert"),
            display
        );

        let unit_quad_data = [
            XyVertex{ position: [-1.0, -1.0] },
            XyVertex{ position: [ 1.0, -1.0] },
            XyVertex{ position: [ 1.0,  1.0] },
            XyVertex{ position: [-1.0,  1.0] }
        ];
        let unit_quad = Rc::new(glium::VertexBuffer::new(display, &unit_quad_data).unwrap());

        ProgramData{
            id_counter: Rc::new(RefCell::new(0)),

            globe_texture,

            globe_gl_buf,

            graticule_gl_buf,

            map_gl_buf,

            cylindrical_lambert_views: vec![],

            gnomonic_views: vec![],

            orthographic_views: vec![],

            stereographic_views: vec![],

            gl_programs: OpenGlPrograms {
                texture_copy_single,
                texture_copy_multi,
                cylindrical_lambert,
                gnomonic,
                orthographic,
                stereographic
            },

            unit_quad,
        }
    }

    pub fn new_unique_id(&self) -> u32 {
        let new_id = *self.id_counter.borrow();
        *self.id_counter.borrow_mut() += 1;

        new_id
    }

    pub fn cylindrical_lambert_views(&mut self) -> &mut Vec<CylindricalLambertView> {
        &mut self.cylindrical_lambert_views
    }

    pub fn gnomonic_views(&mut self) -> &mut Vec<GnomonicView> {
        &mut self.gnomonic_views
    }

    pub fn orthographic_views(&mut self) -> &mut Vec<OrthographicView> {
        &mut self.orthographic_views
    }

    pub fn stereographic_views(&mut self) -> &mut Vec<StereographicView> {
        &mut self.stereographic_views
    }

    pub fn add_cylindrical_lambert_view(&mut self, view: CylindricalLambertView) {
        self.cylindrical_lambert_views.push(view);
    }

    pub fn add_gnomonic_view(&mut self, view: GnomonicView) {
        self.gnomonic_views.push(view);
    }

    pub fn add_orthographic_view(&mut self, view: OrthographicView) {
        self.orthographic_views.push(view);
    }

    pub fn add_stereographic_view(&mut self, view: StereographicView) {
        self.stereographic_views.push(view);
    }
}

fn create_globe_mesh(
    step: cgmath::Deg<f64>,
    display: &glium::Display
) -> LonLatGlBuffers {
    assert!((360.0 / step.0).fract() == 0.0);

    let grid_size_lon = (360.0 / step.0) as usize + 1;
    let grid_size_lat = (180.0 / step.0) as usize - 1;

    let mut vertex_data: Vec<LonLatVertex> = vec![];

    let mut latitude = -90.0 + step.0;
    for _ in 0..grid_size_lat {
        let mut longitude = -180.0;
        for _ in 0..grid_size_lon {
            vertex_data.push(LonLatVertex{ lonlat_position: [longitude as f32, latitude as f32] });
            longitude += step.0;
        }
        latitude += step.0;
    }

    let mut index_data: Vec<u32> = vec![];

    macro_rules! v_index {
        ($i_lon:expr, $i_lat:expr) => { (($i_lon) + ($i_lat) * grid_size_lon) as u32 }
    }

    for i_lon in 0..grid_size_lon {
        for i_lat in 0..grid_size_lat - 1 {
            index_data.push(v_index!(i_lon,     i_lat));
            index_data.push(v_index!(i_lon,     i_lat + 1));
            index_data.push(v_index!(i_lon + 1, i_lat));

            index_data.push(v_index!(i_lon + 1, i_lat));
            index_data.push(v_index!(i_lon,     i_lat + 1));
            index_data.push(v_index!(i_lon + 1, i_lat + 1));
        }
    }

    vertex_data.push(LonLatVertex{ lonlat_position: [0.0, -90.0] }); // south cap
    let s_cap_idx = vertex_data.len() as u32 - 1;
    vertex_data.push(LonLatVertex{ lonlat_position: [0.0,  90.0] }); // north cap
    let n_cap_idx = vertex_data.len() as u32 - 1;

    for i_lon in 0..grid_size_lon {
        // south cap
        index_data.push(v_index!(i_lon, 0));
        index_data.push(v_index!(i_lon + 1, 0));
        index_data.push(s_cap_idx);

        // north cap
        index_data.push(v_index!(i_lon, grid_size_lat - 1));
        index_data.push(v_index!(i_lon + 1, grid_size_lat - 1));
        index_data.push(n_cap_idx);
    }

    let vertices = Rc::new(glium::VertexBuffer::new(display, &vertex_data).unwrap());
    let indices = Rc::new(glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &index_data).unwrap());

    LonLatGlBuffers{ vertices, indices }
}

fn create_graticule(
    step: cgmath::Deg<f64>,
    num_substeps: usize,
    display: &glium::Display
) -> LonLatGlBuffers {
    let mut vertex_data: Vec<LonLatVertex> = vec![];
    let mut index_data: Vec<u32> = vec![];

    let mut longitude = cgmath::Deg(-180.0);
    while longitude <= cgmath::Deg(180.0) {
        let mut latitude = cgmath::Deg(-90.0);
        let mut parallel_starts = true;
        while latitude <= cgmath::Deg(90.0) {
            vertex_data.push(LonLatVertex{ lonlat_position: [longitude.0 as f32, latitude.0 as f32] });
            if !parallel_starts {
                index_data.push((vertex_data.len() - 2) as u32);
                index_data.push((vertex_data.len() - 1) as u32);
            }
            latitude += step / num_substeps as f64;
            parallel_starts = false;
        }

        longitude += step;
    }

    let mut latitude = cgmath::Deg(-90.0);
    while latitude <= cgmath::Deg(90.0) {
        let mut longitude = cgmath::Deg(-180.0);
        let mut meridian_starts = true;
        while longitude <= cgmath::Deg(180.0) {
            vertex_data.push(LonLatVertex{ lonlat_position: [longitude.0 as f32, latitude.0 as f32] });
            if !meridian_starts {
                index_data.push((vertex_data.len() - 2) as u32);
                index_data.push((vertex_data.len() - 1) as u32);
            }
            longitude += step / num_substeps as f64;
            meridian_starts = false;
        }

        latitude += step;
    }

    let vertices = Rc::new(glium::VertexBuffer::new(display, &vertex_data).unwrap());
    let indices = Rc::new(glium::IndexBuffer::new(display, glium::index::PrimitiveType::LinesList, &index_data).unwrap());

    LonLatGlBuffers{ vertices, indices }
}

fn create_texture_from_image(path: &str, display: &glium::Display)
-> glium::texture::texture2d::Texture2d {
    let max_texture_size = display.get_capabilities().max_texture_size as u32;

    let mut map_image = image::open(path).unwrap();

    let dims = map_image.dimensions();
    if dims.0 > max_texture_size || dims.1 > max_texture_size {
        map_image = map_image.resize(
            max_texture_size.min(dims.0),
            max_texture_size.min(dims.1),
            image::imageops::FilterType::CatmullRom
        );
    }
    let dims = map_image.dimensions();

    let img_buffer = match map_image {
        image::DynamicImage::ImageRgb8(image) => image,
        _ => panic!("expected an RGB8 image")
    };

    let layout = img_buffer.as_flat_samples().layout;
    //TODO: handle line padding
    assert!(layout.height_stride == layout.width as usize * layout.channels as usize);

    let texture = glium::texture::texture2d::Texture2d::with_format(
        display,
        glium::texture::RawImage2d{
            data: std::borrow::Cow::<[u8]>::from(img_buffer.as_flat_samples().samples),
            width: dims.0,
            height: dims.1,
            format: glium::texture::ClientFormat::U8U8U8
        },
        glium::texture::UncompressedFloatFormat::U8U8U8,
        glium::texture::MipmapsOption::AutoGeneratedMipmaps
    ).unwrap();

    texture
}

fn create_map_from_shape_file(path: &str, display: &glium::Display)
-> LonLatGlBuffers {
    let mut reader = shapefile::Reader::from_path(path).unwrap();

    let mut vertex_data: Vec<LonLatVertex> = vec![];
    let mut index_data: Vec<u32> = vec![];

    for shape_record in reader.iter_shapes_and_records() {
        let (shape, _record) = shape_record.unwrap();

        // let scalerank: f64 = match record.get("scalerank").unwrap() {
        //     shapefile::dbase::FieldValue::Numeric(value) => value.unwrap(),
        //     _ => panic!("unexpected field type")
        // };

        /*if scalerank == 6.0*/ {
            match shape {
                shapefile::Shape::Polyline(polyline) => {
                    for part in polyline.parts() {
                        for (idx, point) in part.iter().enumerate() {
                            vertex_data.push(LonLatVertex{ lonlat_position: [point.x as f32, point.y as f32] });
                            if idx > 0 {
                                index_data.push((vertex_data.len() - 2) as u32);
                                index_data.push((vertex_data.len() - 1) as u32);
                            }
                        }
                    }
                },
                _ => ()
            }
        }
    }

    let vertices = Rc::new(glium::VertexBuffer::new(display, &vertex_data).unwrap());
    let indices = Rc::new(glium::IndexBuffer::new(display, glium::index::PrimitiveType::LinesList, &index_data).unwrap());

    LonLatGlBuffers{ vertices, indices }
}