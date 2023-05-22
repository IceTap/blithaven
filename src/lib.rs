
//!
//! BlitHaven is a 2-Dimensional Game Engine
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
use std::time::{Duration, Instant};
use glium::glutin::event_loop::{EventLoop, ControlFlow};
use glium::glutin::event::{Event, StartCause};
use glium::*;
use strum_macros::EnumString;

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
    color: [f32; 3]
}
implement_vertex!(Vertex, position, color);


use glium::{Display, index::{PrimitiveType, IndexBuffer}, vertex::VertexBuffer, Program, Frame};
pub struct Batch {
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u16>,
    program: Program,
    display: Display,
    aspect_ratio: f32
}

impl Batch {
    fn new(display: Display, aspect_ratio: f32) -> Self {
        let program = {
                Program::from_source(&display,
                    r#"
                    #version 140
    
                    in vec2 position;
                    in vec3 color;
    
                    out vec4 v_color;
    
                    uniform mat4 matrix;
    
                    void main() {{
                        v_color = vec4(color, 1.0);
                        gl_Position = matrix * vec4(position, 0.0, 1.0);
                    }}
                    "#,
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
        Self { vertex_buffer: vec![], index_buffer: vec![], program: program, display: display, aspect_ratio: aspect_ratio }
    }
    
    pub fn add_quad(&mut self, position: [f32; 2], width: f32, height: f32, color: (f32, f32, f32)) {
        let index_buffer_size = self.vertex_buffer.len() as u16;
        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 1);
        self.index_buffer.push(index_buffer_size + 2);

        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 3);
        self.index_buffer.push(index_buffer_size + 2);

        let color = [color.0, color.1, color.2];
        self.vertex_buffer.push(Vertex { position: position, color: color });
        self.vertex_buffer.push(Vertex { position: [position[0] + width, position[1]], color: color });
        self.vertex_buffer.push(Vertex { position: [position[0] + width, position[1] - height], color: color });
        self.vertex_buffer.push(Vertex { position: [position[0], position[1] - height], color: color });
    }

    fn draw(&self, frame: &mut Frame) {
        let uniforms = uniform! {
            matrix: [
                [self.aspect_ratio, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, -1.0, 1.0f32],
            ]
        };
        let index_buffer = IndexBuffer::new(&self.display, PrimitiveType::TrianglesList, &self.index_buffer).unwrap();
        let vertex_buffer = VertexBuffer::new(&self.display, &self.vertex_buffer).unwrap();

        frame.draw(&vertex_buffer, &index_buffer, &self.program, &uniforms, &Batch::get_default_draw_params()).unwrap();
    }

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
        App {
            batch
        }
    }

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
        (App {
            batch
            }, 
         event_loop
        )
    }

    // pub fn set_world_size(&mut self, size: f32) {
    //     self.scene.distortion = size;
    // }

    pub fn say_hello(&self) {
        println!("Hello from App")
    }

    // pub fn run<F>(&mut self, event_loop: EventLoop<()>, mut input_code: F) where F: 'static + FnMut(&Vec<Event<'_, ()>>) {
    //     crate::run(event_loop, input_code)
    // }

    // pub fn draw_circle_with_shaders(&mut self, position: [f32; 2], radius: f32, vertex: &str, fragment: &str) {
    //     let mut vertecies = vec![position];
    //     let vertex_count: usize = 48;

    //     for i in 0 .. vertex_count {
    //         let x = (i as f32 * PI) / 24.0;
    //         vertecies.push([position[0] + (x).cos() * radius, position[1] + (x).sin() * radius]);
    //     }
    //     vertecies.push([position[0] + radius, position[1]]);

    //     let shape = Shape::new(vertecies, &self.scene.display, None, Some(vertex), Some(fragment), self.scene.distortion, self.aspect_ratio);
    //     // if uniform.is_some() {
    //     //     let uniform = uniform.unwrap();
    //     //     shape.add_uniform(uniform.0, uniform.1);
    //     // }

        
    //     self.scene.add_actor(shape);
    // }

    // pub fn draw_polygon(&mut self, vertecies: Vec<[f32; 2]>, color: (f32, f32, f32, f32)) {
    //     self.scene.add_actor(Shape::new(vertecies, &self.scene.display, Some(color), None, None, self.scene.distortion, self.aspect_ratio));
    // }
    // pub fn draw_polygon_with_shaders(&mut self, vertecies: Vec<[f32; 2]>, vertex: &str, fragment: &str) {
    //     self.scene.add_actor(Shape::new(vertecies, &self.scene.display, None, Some(vertex), Some(fragment), self.scene.distortion, self.aspect_ratio));
    // }

    // pub fn draw_rect(&mut self, position: [f32; 2], width: f32, height: f32, color: (f32,f32,f32,f32)) {
    //     let vertecies = vec![position, [position[0] + width, position[1]], [position[0] + width, position[1] - height], [position[0], position[1] - height]];
    //     self.scene.add_actor(Shape::new(vertecies, &self.scene.display, Some(color), None, None, self.scene.distortion, self.aspect_ratio));
    // }
    // pub fn draw_rect_with_shaders(&mut self, position: [f32; 2], width: f32, height: f32, vertex: &str, fragment: &str) {
    //     let vertecies = vec![position, [position[0] + width, position[1]], [position[0] + width, position[1] - height], [position[0], position[1] - height]];
    //     self.scene.add_actor(Shape::new(vertecies, &self.scene.display, None, Some(vertex), Some(fragment), self.scene.distortion, self.aspect_ratio));
    // }

    // pub fn draw_square(&mut self, position: [f32; 2], size: f32, color: (f32,f32,f32,f32)) {
    //     let vertecies = vec![position, [position[0] + size, position[1]], [position[0] + size, position[1] - size], [position[0], position[1] - size]];
    //     self.scene.add_actor(Shape::new(vertecies, &self.scene.display, Some(color), None, None, self.scene.distortion, self.aspect_ratio));
    // }
    // pub fn draw_square_with_shaders(&mut self, position: [f32; 2], size: f32, vertex: &str, fragment: &str) {
    //     let vertecies = vec![position, [position[0] + size, position[1]], [position[0] + size, position[1] - size], [position[0], position[1] - size]];
    //     self.scene.add_actor(Shape::new(vertecies, &self.scene.display, None, Some(vertex), Some(fragment), self.scene.distortion, self.aspect_ratio));
    // }

    // pub fn draw_circle(&mut self, position: [f32; 2], radius: f32, color: (f32,f32,f32,f32)) {
    //     let mut vertecies = vec![position];
    //     let vertex_count: usize = 48;

    //     for i in 0 .. vertex_count {
    //         let x = (i as f32 * PI) / 24.0;
    //         vertecies.push([position[0] + (x).cos() * radius, position[1] + (x).sin() * radius]);
    //     }
    //     vertecies.push([position[0] + radius, position[1]]);

    //     self.scene.add_actor(Shape::new(vertecies, &self.scene.display, Some(color), None, None, self.scene.distortion, self.aspect_ratio));
    // }



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
        frame.clear_color_and_depth((color.0, color.1, color.2, 0.5), 1.0);
        self.batch.draw(&mut frame);
        self.batch.vertex_buffer = vec![];
        self.batch.index_buffer = vec![];
        frame.finish().unwrap();
    }
}

pub fn new_event_loop() -> EventLoop<()> {
    glium::glutin::event_loop::EventLoop::new()
}

pub fn x_is_pressed(events: &Vec<Event<()>>) -> Option<glium::glutin::event::VirtualKeyCode> {
    for event in events.iter() {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::KeyboardInput { device_id: _, input, is_synthetic } => {
                    if !is_synthetic && input.state == glium::glutin::event::ElementState::Pressed {return Some(input.virtual_keycode.unwrap())}
                },
                _ => (),
            },
            _ => (),
        }
    };
    None
}

#[derive(EnumString)]
enum MyKeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    // TODO: rename
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}