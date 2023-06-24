#![allow(dead_code)]
use earcutr::earcut;
use glium::glutin;
use glium::DrawParameters;
use std::io::Read;
use std::time::Duration;
use std::time::Instant;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event::Event;
use glium::glutin::event::StartCause;
pub mod keycode;

enum Action {
    Stop,
    Continue,
}

pub fn run<F>(event_loop: EventLoop<()>, mut input_code: F)->! where F: 'static + FnMut(&Vec<Event<'_, ()>>) {
    let mut events_buffer = Vec::new();
    let mut next_frame_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        let run_callback = match event.to_static() {
            Some(Event::NewEvents(cause)) => {
                match cause {
                    StartCause::ResumeTimeReached { .. } | StartCause::Init => {
                        true
                    },
                    _ => false
                }
            },
            Some(event) => {
                events_buffer.push(event);
                false
            }
            None => {
                // Ignore this event.
                false
            },
        };

        let action = if run_callback {
            input_code(&events_buffer);
            next_frame_time = Instant::now() + Duration::from_nanos(16666667);
            // TODO: Add back the old accumulator loop in some way
            for event in events_buffer.iter() {
                match event {
                    glutin::event::Event::WindowEvent { event, .. } => match event {
                        glutin::event::WindowEvent::CloseRequested => {
                            *control_flow = glutin::event_loop::ControlFlow::Exit;
                            return
                        },
                        _ => (),
                    },
                    _ => (),
                }
            };

            events_buffer.clear();
            Action::Continue
        } else {
            Action::Continue
        };

        match action {
            Action::Continue => {
                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            },
            Action::Stop => *control_flow = ControlFlow::Exit
        };
    })
}


#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
    style: i32,
    tex_coord: [f32; 2]
}
glium::implement_vertex!(Vertex, position, color, style, tex_coord);

use glium::{Display, index::{PrimitiveType, IndexBuffer}, vertex::VertexBuffer, Program, Frame, Surface};
pub struct Batch {
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u16>,
    program: Program,
    display: Display,
    aspect_ratio: f32,
    distortion: f32,
    positional_offset: [f32; 2]
}

impl Batch {
    fn new(display: Display, aspect_ratio: f32) -> Self {
        use std::fs::File;

        let mut vertex_shader = String::new();
        let _ = File::open(r"./src/shaders/vertex.glsl").expect("Could not open vertex file").read_to_string(&mut vertex_shader);

        let mut fragment_shader = String::new();
        let _ = File::open(r"./src/shaders/fragment.glsl").expect("Could not open fragment file").read_to_string(&mut fragment_shader);

        let program = { Program::from_source(&display, &vertex_shader, &fragment_shader, None).unwrap() };

        Self { vertex_buffer: vec![], index_buffer: vec![], program, display, aspect_ratio, distortion: 1.0, positional_offset: [0.0,0.0] }
    }
    
    fn add_quad(&mut self, position: [f32; 2], width: f32, height: f32, color: (f32, f32, f32), style: i32) {
        let index_buffer_size = self.vertex_buffer.len() as u16;
        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 1);
        self.index_buffer.push(index_buffer_size + 2);

        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 3);
        self.index_buffer.push(index_buffer_size + 2);

        let color = [color.0, color.1, color.2];
        self.vertex_buffer.push(Vertex { position, color, style, tex_coord: [0.0,1.0] });
        self.vertex_buffer.push(Vertex { position: [position[0] + width, position[1]], color, style, tex_coord: [1.0,1.0] });
        self.vertex_buffer.push(Vertex { position: [position[0] + width, position[1] - height], color, style, tex_coord: [1.0, 0.0] });
        self.vertex_buffer.push(Vertex { position: [position[0], position[1] - height], color, style, tex_coord: [0.0,0.0] });
    }

    // Pushing an n sided polygon to the batch
    // I'm not sure about the performance impact of this function 
    // texture coordinates do not work
    pub fn add_polygon(&mut self, points: Vec<f64>, color: (f32, f32, f32)) {
        let index_buffer_initial_size = self.vertex_buffer.len();
        let shape_index_buffer = earcut(&points, &[], 2);


        for index in shape_index_buffer.unwrap_or_default().iter() {
            self.index_buffer.push((index_buffer_initial_size + index) as u16)
        }
        
        let color = [color.0, color.1, color.2];
        let mut current_point = [0.0,0.0];
        for (index, point) in points.iter().enumerate() {
            if index % 2 == 0 {
                current_point[0] = *point as f32;
            }
            else {
                current_point[1] = *point as f32;
                self.vertex_buffer.push(Vertex { position: current_point, color, style: 0, tex_coord: [0.0,0.0] });
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let uniforms = glium::uniform! {
            matrix: [
                [self.aspect_ratio / self.distortion, 0.0, 0.0, 0.0],
                [0.0, 1.0 / self.distortion, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [self.positional_offset[0], self.positional_offset[1], -1.0, 1.0f32],
            ]
        };
        let index_buffer = IndexBuffer::new(&self.display, PrimitiveType::TrianglesList, &self.index_buffer).unwrap();
        let vertex_buffer = VertexBuffer::new(&self.display, &self.vertex_buffer).unwrap();
        
        frame.draw(&vertex_buffer, &index_buffer, &self.program, &uniforms, &Batch::get_default_draw_params()).unwrap(); }

    fn get_default_draw_params() -> DrawParameters<'static> {
        glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            polygon_mode: glium::PolygonMode::Fill,
            .. Default::default()
        }
    }
    
}

pub struct App {
    pub batch: Batch,
}

impl App {
    pub fn init(title: &str, event_loop: &EventLoop<()>) -> Self {
        let mut icon: Vec<u8> = vec![];
        let mut counter = 0;

        for i in 0 .. 256 * 4 {
            if counter == 3 {
                icon.push(255);
                counter = 0;
            }

            else {
                if i % 2 == 0 {
                    icon.push(200)
                }
                else if i % 3 == 0 {
                    icon.push(150);
                } 
                
                else {icon.push(60)}
                counter += 1
            }
        }
        let icon = glutin::window::Icon::from_rgba(icon, 16, 16).expect("BADICON");

        let window = glutin::window::WindowBuilder::new().with_title(title).with_window_icon(Some(icon));
        let context_buffer = glutin::ContextBuilder::new().with_depth_buffer(24);
        let batch = Batch::new(glium::Display::new(window, context_buffer, &event_loop).unwrap(), 16.0 / 9.0);
        return App { batch }
    }

    // Recommended
    pub fn init_with_loop(title: &str) -> (Self, EventLoop<()>) {
        let mut icon: Vec<u8> = vec![];
        let mut counter = 0;

        for i in 0 .. 256 * 4 {
            if counter == 3 {
                icon.push(255);
                counter = 0;
            }

            else {
                if i % 2 == 0 {
                    icon.push(200)
                }
                else if i % 3 == 0 {
                    icon.push(150);
                } 
                
                else {icon.push(60)}
                counter += 1
            }
        }
        let icon = glutin::window::Icon::from_rgba(icon, 16, 16).expect("BADICON");

        let event_loop = new_event_loop();
        let window = glutin::window::WindowBuilder::new().with_title(title).with_window_icon(Some(icon));
        let context_buffer = glutin::ContextBuilder::new().with_depth_buffer(24);
        let batch = Batch::new(glium::Display::new(window, context_buffer, &event_loop).unwrap(), 16.0 / 9.0);
        return ( App { batch }, event_loop )
    }

    pub fn set_zoom(&mut self, size: f32) {
        self.batch.distortion = size;
    }

    pub fn set_positional_offset(&mut self, offset: [f32; 2]) {
        self.batch.positional_offset = offset;
    }

    pub fn add_to_positional_offset(&mut self, offset: [f32; 2]) {
        self.batch.positional_offset[0] += offset[0];
        self.batch.positional_offset[1] += offset[1];
    }

    pub fn say_hello(&self) {
        println!("Hello from App")
    }

    pub fn quad(&mut self, position: [f32; 2], width: f32, height: f32, color: (f32, f32, f32)) {
        self.batch.add_quad(position, width, height, color, 0);
    }

    pub fn circle(&mut self, position: [f32; 2], radius: f32, color: (f32,f32,f32)) {
        self.batch.add_quad([position[0] - radius / 2.0, position[1] + radius / 2.0 ], radius, radius, color, 1)
    }
    
    pub fn square(&mut self, position: [f32; 2], size: f32, color: (f32,f32,f32)) {
        self.batch.add_quad(position, size, size, color, 0)
    }

    pub fn save_frame(&mut self, color: (f32,f32,f32), events: &Vec<Event<()>>) {
        for event in events.iter() {
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                        glutin::event::WindowEvent::Resized(u) => {
                            self.batch.aspect_ratio = u.height as f32 / u.width as f32 ;
                        }
                    _ => (),
                },
                _ => (),
            }
        };
        // self.batch.finish(color);
        let mut frame = self.batch.display.draw();
        frame.clear_color_and_depth((color.0, color.1, color.2, 1.0), 1.0);
        self.batch.draw(&mut frame);
        self.batch.vertex_buffer = vec![];
        self.batch.index_buffer = vec![];
        frame.finish().unwrap();
    }
}


pub fn new_event_loop() -> EventLoop<()> {
    glium::glutin::event_loop::EventLoop::new()
}

pub fn key_pressed(events: &Vec<Event<()>>) -> String {
    for event in events.iter() {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::KeyboardInput { device_id: _, input, is_synthetic } => {
                    if !is_synthetic && input.state == glium::glutin::event::ElementState::Pressed {return format!("{:?}", input.virtual_keycode.unwrap())}
                },
                _ => (),
            },
            _ => (),
        }
    };
    String::new()
}
