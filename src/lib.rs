
//!
//! BlitHaven is a fast, high performance 2-Dimensional Game Engine
//! 
//! 
 
//! # Example
//! 
//! ```
//! 
//! use blithaven::*;
//! 
//! fn main() {
//!     
//!     let mut app = App::init("test");
//! 
//!     blithaven::start_loop(app.event_loop, move | _events | {
//!         app.scene.draw_polygon(vec![[0.0,0.0], [0.5,0.0], [0.5,0.5]], (1.0,0.0,0.0,1.0));
//! 
//!         app.scene.save_frame((0.2,0.2,0.2)).finish().unwrap();
//!         Action::Continue
//!     });
//! }
//! 
//! ```


#![allow(dead_code)]
use glium::{glutin, DrawParameters};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use glium::glutin::event_loop::{EventLoop, ControlFlow};
use glium::glutin::event::{Event, StartCause};
use glium::*;
use earcutr;

pub enum Action {
    Stop,
    Continue,
}

static mut ASPECT_RATIO: f32 = 1.0;


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
                            return;
                        },
                        glutin::event::WindowEvent::Resized(u) => {
                            unsafe { ASPECT_RATIO = u.height as f32 / u.width as f32 };
                        }
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
        }

    })
}

pub fn get_params_defualt() -> DrawParameters<'static> {
    glium::DrawParameters {
        blend: glium::draw_parameters::Blend::alpha_blending(),
        polygon_mode: glium::PolygonMode::Fill,
        .. Default::default()
    }
}




#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 2]
}
implement_vertex!(Vertex, position);


use glium::Display;
use glium::uniforms::{UniformsStorage, EmptyUniforms};
use glium::index::PrimitiveType;
use glium::index::IndexBuffer;
use glium::vertex::VertexBuffer;
use glium::Program;
use glium::Frame;
use std::vec::Vec;

pub struct Shape {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    program: Program,
}

impl Shape {
    fn new_shape(vertices: Vec<[f32; 2]>, display: &Display, color: (f32, f32, f32, f32)) -> Self {
        assert!(vertices.len() > 2, "Number of vertices in polygon must be greater than 2.");
        let mut verts = Vec::<f32>::new();
        let vertices = {
            let mut v: Vec<Vertex> = Vec::new();
            for vertex in vertices {
                v.push(
                    Vertex { position: [vertex[0] / 400.0, vertex[1] / 400.0] }
                );
                verts.push(vertex[0]);
                verts.push(vertex[1]);
            }
            VertexBuffer::new(display, &v).unwrap()
        };
        let indices = {
            let mut data = Vec::<u16>::new();
            for num in earcutr::earcut(&verts, &vec![], 2).unwrap() {
                data.push(num as u16)
            };

            IndexBuffer::new(display, PrimitiveType::TrianglesList, &data).unwrap()
        };
        let program = {
            Program::from_source(display,
                &format!(r#"
                #version 140

                in vec2 position;

                out vec4 v_color;

                uniform mat4 matrix;

                void main() {{
                    v_color = vec4({}, {}, {}, {});
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                }}
                "#, color.0, color.1, color.2, color.3),
                r#"
                    #version 140

                    in vec4 v_color;

                    out vec4 f_color;

                    void main() {

                        f_color = v_color;
                    }
                "#,
                None
            ).unwrap()
        };
        
        Shape { vertex_buffer: vertices, index_buffer: indices, program: program }
    }

    pub fn new(vertices: Vec<[f32; 2]>, display: &Display, vertex: &str, fragment: &str) -> Self {
        assert!(vertices.len() > 2, "Number of vertices in polygon must be greater than 2.");
        let mut verts = Vec::<f32>::new();
        let vertices = {
            let mut v: Vec<Vertex> = Vec::new();
            for vertex in vertices {
                v.push(
                    Vertex { position: [vertex[0] / 400.0, vertex[1] / 400.0] }
                );
                verts.push(vertex[0]);
                verts.push(vertex[1]);
            }
            VertexBuffer::new(display, &v).unwrap()
        };
        let indices = {
            let mut data = Vec::<u16>::new();
            for num in earcutr::earcut(&verts, &vec![], 2).unwrap() {
                data.push(num as u16)
            };

            IndexBuffer::new(display, PrimitiveType::TrianglesList, &data).unwrap()
        };
        let program = {
            Program::from_source(display,
                vertex,
                fragment,
                None
            ).unwrap()
        };
        
        Shape { vertex_buffer: vertices, index_buffer: indices, program: program }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &Scene::get_uniforms(), &get_params_defualt()).unwrap();
    }
}

pub struct Scene {
    actors: Vec<Shape>,
    pub display: Display,
}

impl Scene {
    fn get_uniforms() -> UniformsStorage<'static, [[f32; 4]; 4], EmptyUniforms> {
        return uniform! {
            matrix: [
                [unsafe { ASPECT_RATIO }, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, -1.0, 1.0f32],
            ]
        };
    }

    pub fn add_actor(&mut self, shape: Shape) {
        self.actors.push(shape)
    }

    pub fn save_frame(&mut self, clear_color: (f32, f32, f32)) {
        let mut frame = self.display.draw();
        frame.clear_color_and_depth((clear_color.0, clear_color.1, clear_color.2, 0.5), 1.0);
        for actor in self.actors.iter() {
            actor.draw(&mut frame);
        }
        self.actors = Vec::new();
        frame.finish().unwrap();
    }



    pub fn draw_polygon(&mut self, vertices: Vec<[f32; 2]>, color: (f32, f32, f32, f32)) {
        self.add_actor(Shape::new_shape(vertices, &self.display, color));
    }
    pub fn draw_polygon_with_shaders(&mut self, vertices: Vec<[f32; 2]>, vertex: &str, fragment: &str) {
        self.add_actor(Shape::new(vertices, &self.display, vertex, fragment ));
    }



    pub fn draw_rect(&mut self, position: [f32; 2], width: f32, height: f32, color: (f32,f32,f32,f32)) {
        let vertecies = vec![position, [position[0] + width, position[1]], [position[0] + width, position[1] - height], [position[0], position[1] - height]];
        self.add_actor(Shape::new_shape(vertecies, &self.display, color));
    }
    pub fn draw_rect_with_shaders(&mut self, position: [f32; 2], width: f32, height: f32, vertex: &str, fragment: &str) {
        let vertecies = vec![position, [position[0] + width, position[1]], [position[0] + width, position[1] - height], [position[0], position[1] - height]];
        self.add_actor(Shape::new(vertecies, &self.display, vertex, fragment));
    }



    pub fn draw_square(&mut self, position: [f32; 2], size: f32, color: (f32,f32,f32,f32)) {
        let vertecies = vec![position, [position[0] + size, position[1]], [position[0] + size, position[1] - size], [position[0], position[1] - size]];
        self.add_actor(Shape::new_shape(vertecies, &self.display, color));
    }
    pub fn draw_square_with_shaders(&mut self, position: [f32; 2], size: f32, vertex: &str, fragment: &str) {
        let vertecies = vec![position, [position[0] + size, position[1]], [position[0] + size, position[1] - size], [position[0], position[1] - size]];
        self.add_actor(Shape::new(vertecies, &self.display, vertex, fragment));
    }

    

    pub fn draw_circle(&mut self, position: [f32; 2], radius: f32, color: (f32,f32,f32,f32)) {
        let mut vertecies = vec![position];
        let vertex_count: usize = 48;

        for i in 0 .. vertex_count {
            let x = (i as f32 * PI) / 12.0;
            vertecies.push([position[0] + (x).cos() * radius, position[1] + (x).sin() * radius]);
        }vertecies.push([position[0] + radius, position[1]]);

        self.add_actor(Shape::new_shape(vertecies, &self.display, color));
    }
    pub fn draw_circle_with_shaders(&mut self, position: [f32; 2], radius: f32, vertex: &str, fragment: &str) {
        let mut vertecies = vec![position];
        let vertex_count: usize = 48;

        for i in 0 .. vertex_count {
            let x = (i as f32 * PI) / 12.0;
            vertecies.push([position[0] + (x).cos() * radius, position[1] + (x).sin() * radius]);
        }vertecies.push([position[0] + radius, position[1]]);

        self.add_actor(Shape::new(vertecies, &self.display, vertex, fragment));
    }
    
}



pub struct App {
    //pub display: Display,
    pub scene: Scene,
    pub event_loop: EventLoop<()>,
    
}

impl App {
    pub fn init(title: &str) -> Self {
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

        let event_loop = glutin::event_loop::EventLoop::new();
        let window = glutin::window::WindowBuilder::new().with_title(title).with_window_icon(Some(icon));
        let context_buffer = glutin::ContextBuilder::new().with_depth_buffer(24);
        App {scene: Scene { actors: Vec::new(), display: glium::Display::new(window, context_buffer, &event_loop).unwrap() }, event_loop: event_loop }
    }
}

pub fn circle(scene: &mut Scene, position: [f32; 2],radius: f32, color: (f32,f32,f32,f32)) {
    scene.draw_circle(position, radius, color);
}
pub fn rect(scene: &mut Scene, position: [f32; 2], width: f32, height: f32, color: (f32,f32,f32,f32)) {
    scene.draw_rect(position, width, height, color);
}
pub fn square(scene: &mut Scene, position: [f32; 2], size: f32, color: (f32,f32,f32,f32)) {
    scene.draw_square(position, size, color);
}
pub fn polygon_from(scene: &mut Scene, vertecies: Vec<[f32; 2]>, color: (f32,f32,f32,f32)) {
    scene.draw_polygon(vertecies, color);
}
pub fn polygon(scene: &mut Scene, vertecies: Vec<f32>, color: (f32,f32,f32,f32)) {
    let vertecies = {
        let mut verts: Vec<[f32;2]> = vec![];
        for index in 0 .. vertecies.len() {
            if index % 2 == 0 {
                verts.push([vertecies[index], vertecies[index + 1]])
            }
        }
        verts
    };
    scene.draw_polygon(vertecies, color);
}