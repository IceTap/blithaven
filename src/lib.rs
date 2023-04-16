
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

/// A struct representing a shape in a 2D space.
pub struct Shape {
    position: [f32; 2],
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    program: Program,
}

impl Shape {
    /// Returns the uniforms used for rendering the shape.
    ///
    /// The uniforms consist of a matrix used for transforming the shape, which includes
    /// the aspect ratio, position, and other parameters.
    fn get_uniforms(&self) -> UniformsStorage<'static, [[f32; 4]; 4], EmptyUniforms> {
        return uniform! {
            matrix: [
                [unsafe { ASPECT_RATIO }, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [self.position[0], self.position[1], -1.0, 1.0f32],
            ]
        };
    }

    /// Creates a new shape with the given parameters.
    ///
    /// The shape is created with a position, a list of vertices, a display object, and a color.
    ///
    /// # Panics
    ///
    /// This function will panic if the number of vertices is less than 3.
    fn new_shape(position: [f32; 2], vertices: Vec<[f32; 2]>, display: &Display, color: (f32, f32, f32, f32)) -> Self {
        assert!(vertices.len() > 2, "Number of vertices must be greater than 2.");
        let mut verts = Vec::<f32>::new();
        let vertices = {
            let mut v: Vec<Vertex> = Vec::new();
            for vertex in vertices {
                v.push(
                    Vertex { position: vertex }
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
        
        Shape { position: position, vertex_buffer: vertices, index_buffer: indices, program: program }
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
    /// Adds an actor with the specified shape to the scene.
    ///
    /// # Arguments
    ///
    /// * `shape` - The shape of the actor to be added to the scene.
    ///
    ///
    pub fn add_actor(&mut self, shape: Shape) {
        self.actors.push(shape)
    }

    /// Saves the current frame of the scene to a `Frame` object.
    ///
    /// # Arguments
    ///
    /// * `clear_color` - The color to be used for clearing the frame's color and depth buffers.
    ///                   It should be specified as a tuple of four f32 values representing RGBA.
    ///
    /// # Returns
    ///
    /// The `Frame` object representing the saved frame.
    ///
    ///
    pub fn save_frame(&mut self, clear_color: (f32, f32, f32)) -> Frame {
        let mut frame = self.display.draw();
        frame.clear_color_and_depth((clear_color.0, clear_color.1, clear_color.2, 1.0), 1.0);
        for actor in self.actors.iter() {
            actor.draw(&mut frame);
        }
        self.actors = Vec::new();
        frame
    }

    /// Draws a polygon shape on the scene.
    ///
    /// # Arguments
    ///
    /// * `vertices` - The vertices of the polygon shape. Should be specified as a vector of arrays
    ///                 with two f32 values representing the x and y coordinates of each vertex.
    /// * `color` - The color of the polygon shape. Should be specified as a tuple of four f32 values
    ///             representing RGBA.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut app = App::init("Example Polygon");
    /// 
    /// let vertices = vec![[0.0, 0.0], [1.0, 1.0], [-1.0, 1.0]];
    /// app.scene.draw_polygon(vertices, (0.0, 1.0, 0.0, 1.0));
    /// // The polygon shape with the specified vertices and color will be drawn on the scene.
    /// ```
    ///
    pub fn draw_polygon(&mut self, vertices: Vec<[f32; 2]>, color: (f32, f32, f32, f32)) {
        self.add_actor(Shape::new_shape(vertices[0], vertices, &self.display, color));
    }

    /// Add a rectangle shape to the scene.
    ///
    /// # Arguments
    ///
    /// * `position` - The position of the rectangle as a `[f32; 2]` array representing the x and y coordinates.
    /// * `width` - The width of the rectangle as a `f32` value.
    /// * `height` - The height of the rectangle as a `f32` value.
    /// * `color` - The color of the rectangle as a tuple of `(f32,f32,f32,f32)` representing RGBA values.
    ///
    /// # Example
    ///
    /// ```
    /// let mut scene = Scene::new();
    /// scene.draw_rect([0.0, 0.0], 100.0, 50.0, (1.0, 0.0, 0.0, 1.0));
    /// ```
    pub fn draw_rect(&mut self, position: [f32; 2], width: f32, height: f32, color: (f32,f32,f32,f32)) {
        let vertecies = vec![position, [position[0] + width, position[1]], [position[0] + width, position[1] - height], [position[0], position[1] - height]];
        self.add_actor(Shape::new_shape(position, vertecies, &self.display, color));
    }

    /// Add a square shape to the scene.
    ///
    /// # Arguments
    ///
    /// * `position` - The position of the square as a `[f32; 2]` array representing the x and y coordinates.
    /// * `size` - The size of the square as a `f32` value.
    /// * `color` - The color of the square as a tuple of `(f32,f32,f32,f32)` representing RGBA values.
    ///
    /// # Example
    ///
    /// ```
    /// let mut scene = Scene::new();
    /// scene.draw_square([50.0, 50.0], 100.0, (0.0, 1.0, 0.0, 1.0));
    /// ```
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
    /// Initialize the app with scene and eventloop.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the created window.
    ///
    /// # Example
    ///
    /// ```
    /// let mut app = App::init("Example window");
    /// ```
    pub fn init(title: &str) -> Self {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window = glutin::window::WindowBuilder::new().with_title(title);
        let context_buffer = glutin::ContextBuilder::new().with_depth_buffer(24);
        App {scene: Scene { actors: Vec::new(), display: glium::Display::new(window, context_buffer, &event_loop).unwrap() }, event_loop: event_loop }
    }
}
