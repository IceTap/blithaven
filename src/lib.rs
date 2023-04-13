
//!
//! BlitHaven is a fast, high performance 2-Dimensional Game Engine
//! 
//! 
 
//! # Examples
//! 
//! ```
//! 
//! use blithaven::*;
//! 
//! blithaven::start_loop(app.event_loop, move | _events | {
//!     app.scene.draw_polygon(vec![[0.0,0.0], [0.5,0.0], [0.5,0.5]], (1.0,0.0,0.0));
//!     
//!     app.scene.save_frame((0.2,0.2,0.2)).finish().unwrap();
//!     Action::Continue
//! });
//! 
//! ```




#![allow(dead_code)]
use glium::{glutin, DrawParameters};
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

/// # Examples
/// 
/// ```
/// blithaven::start_loop(app.event_loop, move | _events | {     
///     app.scene.save_frame((0.2,0.2,0.2)).finish().unwrap();
///     Action::Continue
/// });
/// ```
pub fn start_loop<F>(event_loop: EventLoop<()>, mut callback: F)->! where F: 'static + FnMut(&Vec<Event<'_, ()>>) -> Action {
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
            let action = callback(&events_buffer);
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
            action
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
        // depth: glium::Depth {
        //     test: glium::draw_parameters::DepthTest::IfLess,
        //     write: true,
        //     .. Default::default()
        // },
        blend: glium::draw_parameters::Blend::alpha_blending(),
        //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        polygon_mode: glium::PolygonMode::Fill,
        .. Default::default()
    }
}




#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 2]
}
implement_vertex!(Vertex, position);


pub struct Shape {
    pub position: [f32; 2],
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    program: Program
}

impl Shape {
    pub fn get_uniforms(&self) -> glium::uniforms::UniformsStorage<'static, [[f32; 4]; 4], glium::uniforms::EmptyUniforms> {
        return uniform! {
            matrix: [
                [unsafe {ASPECT_RATIO}, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [self.position[0], self.position[1], -1.0, 1.0f32],
            ]
        };
    }

    pub fn new_shape(position: [f32; 2], vertecies: Vec<[f32; 2]>, display: &Display, color: (f32,f32,f32,f32)) -> Self {
        assert!(vertecies.len() > 2);
        let mut verts = Vec::<f32>::new();
        let vertecies = {
            let mut v: Vec<Vertex> = Vec::new();
            for vertex in vertecies {
                v.push(
                    Vertex { position: vertex }
                );
                verts.push(vertex[0]);
                verts.push(vertex[1]);
            }
            glium::VertexBuffer::new(display, &v).unwrap()
        };
        let indicies = {
            let mut data = Vec::<u16>::new();
            for num in earcutr::earcut(&verts, &vec![],2).unwrap() {
                data.push(num as u16)
            };

            glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &data).unwrap()
        };
        let program = {
            glium::Program::from_source(display, 
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
        
        Shape { position: position, vertex_buffer: vertecies, index_buffer: indicies, program: program }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &self.get_uniforms(), &get_params_defualt()).unwrap();
    }
}

pub struct Scene {
    actors: Vec<Shape>,
    display: Display,
}

impl Scene {
    
    pub fn add_actor(&mut self, shape: Shape) {
        self.actors.push(shape)
    }

    pub fn save_frame(&mut self, clear_color: (f32,f32,f32)) -> Frame {
        let mut frame = self.display.draw();
        frame.clear_color_and_depth((clear_color.0, clear_color.1, clear_color.2, 1.0), 1.0);
        for actor in self.actors.iter() {
            actor.draw(&mut frame,);
        }
        self.actors = Vec::new();
        frame
        
    }

    pub fn draw_polygon(&mut self, vertecies: Vec<[f32; 2]>, color: (f32,f32,f32,f32)) {
        self.add_actor(Shape::new_shape(vertecies[0], vertecies, &self.display, color));
    }

    pub fn draw_rect(&mut self, position: [f32; 2], width: f32, height: f32, color: (f32,f32,f32,f32)) {
        let vertecies = vec![position, [position[0] + width, position[1]], [position[0] + width, position[1] - height], [position[0], position[1] - height]];
        self.add_actor(Shape::new_shape(position, vertecies, &self.display, color));
    }

    pub fn draw_square(&mut self, position: [f32; 2], size: f32, color: (f32,f32,f32,f32)) {
        let vertecies = vec![position, [position[0] + size, position[1]], [position[0] + size, position[1] - size], [position[0], position[1] - size]];
        self.add_actor(Shape::new_shape(position, vertecies, &self.display, color));
    }
}



pub struct App {
    //pub display: Display,
    pub scene: Scene,
    pub event_loop: EventLoop<()>,
    
}

impl App {
    pub fn init(title: &str) -> Self {
        let mut event_loop = glutin::event_loop::EventLoopBuilder::new();
        let window = glutin::window::WindowBuilder::new().with_title(title);
        let context_buffer = glutin::ContextBuilder::new().with_depth_buffer(24);
        App {scene: Scene { actors: Vec::new(), display: glium::Display::new(window, context_buffer, &event_loop.build()).unwrap() }, event_loop: event_loop.build() }
    }
}




// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//     }
// }
