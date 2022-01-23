//
// Map Projections
// Copyright (c) 2022 Filip Szczerek <ga.software@yahoo.com>
//
// This project is licensed under the terms of the MIT license
// (see the LICENSE file for details).
//

use glium::Surface;
use std::cell::RefCell;
use std::rc::Rc;

mod clipboard_support;

pub struct Runner {
    event_loop: glium::glutin::event_loop::EventLoop<()>,
    display: glium::Display,
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: Rc<RefCell<imgui_glium_renderer::Renderer>>
}

pub fn create_runner(logical_font_size: f64) -> Runner {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let context = glium::glutin::ContextBuilder::new().with_vsync(true);
    let builder = glium::glutin::window::WindowBuilder::new()
        .with_title("Projections".to_owned())
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1280f64, 768f64));
    let display =
        glium::Display::new(builder, context, &event_loop).expect("Failed to initialize display.");

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    if let Some(backend) = clipboard_support::init() {
        imgui.set_clipboard_backend(backend);
    } else {
        eprintln!("Failed to initialize clipboard.");
    }

    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), &window, imgui_winit_support::HiDpiMode::Default);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (logical_font_size * hidpi_factor) as f32;

    imgui.fonts().add_font(&[
        imgui::FontSource::TtfData {
            data: include_bytes!(
                "../resources/fonts/NotoSans-Regular.ttf"
            ),
            size_pixels: font_size,
            config: Some(imgui::FontConfig {
                glyph_ranges: imgui::FontGlyphRanges::from_slice(&[
                    0x0020, 0x00FF, // Basic Latin, Latin-1 Supplement
                    0
                ]),
                ..imgui::FontConfig::default()
            }),
        },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    imgui.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
    imgui.io_mut().config_windows_move_from_title_bar_only = true;

    let renderer = imgui_glium_renderer::Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer.");

    Runner {
        event_loop,
        display,
        imgui,
        platform,
        renderer: Rc::new(RefCell::new(renderer))
    }
}

impl Runner {
    pub fn renderer(&self) -> &Rc<RefCell<imgui_glium_renderer::Renderer>> {
        &self.renderer
    }

    pub fn platform(&self) -> &imgui_winit_support::WinitPlatform {
        &self.platform
    }

    pub fn display(&self) -> &glium::Display {
        &self.display
    }

    pub fn main_loop<F>(self, mut run_ui: F)
        where F: FnMut(&mut bool, &mut imgui::Ui, &glium::Display, &Rc<RefCell<imgui_glium_renderer::Renderer>>) + 'static
    {
        let Runner {
            event_loop,
            display,
            mut imgui,
            mut platform,
            renderer,
            ..
        } = self;

        let mut last_frame = std::time::Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            glium::glutin::event::Event::NewEvents(_) => {
                let now = std::time::Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            },

            glium::glutin::event::Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui.io_mut(), &gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            },

            glium::glutin::event::Event::RedrawRequested(_) => {
                let mut ui = imgui.frame();

                let mut run = true;
                run_ui(&mut run, &mut ui, &display, &renderer);
                if !run {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                }

                let gl_window = display.gl_window();
                let mut target = display.draw();
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                platform.prepare_render(&ui, gl_window.window());
                let draw_data = imgui.render();
                renderer.borrow_mut()
                    .render(&mut target, draw_data)
                    .expect("Rendering failed.");
                target.finish().expect("Failed to swap buffers.");
            },

            glium::glutin::event::Event::WindowEvent {
                event: glium::glutin::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,

            event => {
                let converted_event = convert_touch_to_mouse(event);

                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &converted_event);
            }
        })
    }
}

fn convert_touch_to_mouse<'a, T>(event: glium::glutin::event::Event<'a, T>) -> glium::glutin::event::Event<'a, T> {
    use glium::glutin::event;

    match event {
        event::Event::WindowEvent {
            window_id,
            event: event::WindowEvent::Touch(touch),
        } => {
            //TODO: do something better here, e.g. remember the last seen mouse device id
            let device_id = touch.device_id.clone();

            match touch.phase {
                event::TouchPhase::Started => event::Event::WindowEvent{
                    window_id: window_id.clone(),
                    event: event::WindowEvent::MouseInput{
                        device_id,
                        state: event::ElementState::Pressed,
                        button: event::MouseButton::Left,
                        modifiers: Default::default()
                    },
                },

                event::TouchPhase::Ended => event::Event::WindowEvent{
                    window_id: window_id.clone(),
                    event: event::WindowEvent::MouseInput{
                        device_id,
                        state: event::ElementState::Released,
                        button: event::MouseButton::Left,
                        modifiers: Default::default()
                    },
                },

                _ => event
            }
        },

        _ => event
    }
}