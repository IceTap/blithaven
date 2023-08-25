use std::{time::{Instant, Duration}, fs::File, io::Read};

use glium::{glutin::{event_loop::{EventLoop, ControlFlow}, event::{Event, StartCause}, window::WindowBuilder, dpi::{Size, LogicalSize}, ContextBuilder}, implement_vertex, Display, IndexBuffer, VertexBuffer, index::PrimitiveType, Program, Surface, DrawParameters, Frame};
use glium::glutin;

enum Action {
    Continue
}

struct MakeLightBulbGoAway;

pub fn start<F>(event_loop: EventLoop<()>, mut input_code: F)->! where F: 'static + FnMut(&Vec<Event<'_, ()>>) {
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
            next_frame_time = Instant::now() + Duration::from_nanos(1666667);
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
            }
        };
    })
}


pub struct App {
    pub options: Options,
    display: Display,
    pub batch: Batch,
    texture_batches: Vec<TextureBatch>,
    last_frame_time: Instant,
    last_fps_output: Instant,
}

impl App {
    pub fn new(title: &str, window_width: u32, window_height: u32) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new().with_title(title).with_inner_size(Size::from(LogicalSize::new(window_width, window_height))).with_resizable(false);
        let context_buffer = ContextBuilder::new().with_depth_buffer(25);

        let display = Display::new(window_builder, context_buffer, &event_loop).unwrap();
        let batch = Batch::new(&display, window_width as i32, window_height as i32);

        ( App { display, batch, options: Options::new(window_width as i32, window_height as i32), texture_batches: Vec::new(), last_frame_time: Instant::now(), last_fps_output: Instant::now() }, event_loop)
    }

    pub fn finish(&mut self, clear_color: [f32; 3], events: &Vec<Event<()>>) {
        for event in events.iter() {
            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                        glutin::event::WindowEvent::Resized(u) => {
                            self.batch.window_height = u.height as i32;
                            self.batch.window_width = u.width as i32;
                            self.options.window_height = u.height as i32;
                            self.options.window_width = u.width as i32;
                        }
                    _ => (),
                },
                _ => (),
            }
        };

        let mut frame = self.display.draw();
        frame.clear_color_and_depth((clear_color[0], clear_color[1], clear_color[2], 1.0), 1.0);
        self.batch.draw(&mut frame, &self.display);
        for texture_batch in self.texture_batches.iter_mut() {
            texture_batch.draw(&mut frame, &self.display);
        }
        frame.finish().unwrap();

        if self.last_fps_output.elapsed() > Duration::from_secs(1) {
            println!("{:?}", 1.0 / (self.last_frame_time.elapsed().as_secs_f64()));
            self.last_fps_output = Instant::now();
        }
        self.last_frame_time = Instant::now();
    }

    pub fn triangle(&mut self, p1: [i32; 2], p2: [i32; 2], p3: [i32; 2], color: (f32, f32, f32)) {
        self.batch.raw_quad(p1, p2, p3, p1, color, 0, self.options.use_pixel_space, 0.0)
    }

    pub fn rect(&mut self, position: [i32; 2], width: i32, height: i32, color: (f32, f32, f32)) {
        self.batch.add_quad(position, width, height, color, 0, self.options.use_pixel_space, 0.0)
    }

    pub fn circle(&mut self, position: [i32; 2], radius: i32, color: (f32, f32, f32)) {
        self.batch.add_quad(
            [position[0] - radius, position[1] - radius], 
            radius * 2, 
            radius * 2, 
            color, 1, self.options.use_pixel_space, 0.0)
    }

    pub fn square(&mut self, position: [i32; 2], size: i32, color: (f32, f32, f32)) {
        self.batch.add_quad(
            position,
            size,
            size,
            color,
            0,
            self.options.use_pixel_space, 0.0);
    }

    pub fn line(&mut self, p1: [i32; 2], p2: [i32; 2], width: i32, color: (f32, f32, f32)) {
        self.batch.add_quad(
            p1, p2[0] - p1[0], p2[1] - p1[1], color, 2, self.options.use_pixel_space, width as f32 / 1000.0)

    }

    // just a wrapper for drawing sets of triangles using the triangle function
    pub fn polygon(&mut self, points: Vec<i32>, color: (f32, f32, f32)) {
        let points : Vec<f32> = points.iter().map(|&x| x as f32).collect();
        let indecies = earcutr::earcut(&points, &vec![], 2).unwrap();
        let points: Vec<[i32; 2]> = {
            let mut new_points: Vec<[i32; 2]> = Vec::new();
            let mut current_point = [0,0];
            for i in points.iter().enumerate() {
                if i.0 % 2 == 0 {
                    current_point[0] = *i.1 as i32;
                }
                else {
                    current_point[1] = *i.1 as i32;
                    new_points.push(current_point);
                }
            }
            new_points
        };

        for index in 0 .. indecies.len() {
            if (index + 1) % 3 == 0 {
                self.triangle(points[indecies[index - 2]], points[indecies[index - 1]], points[indecies[index]], color)
            }
        }
    }

    pub fn texture_quad(&mut self, position: [i32; 2], width: i32, height: i32, texture_path: &str) {
        for texture_batch in self.texture_batches.iter_mut() {
            if texture_path == texture_batch.path {
                texture_batch.add_quad(position, width, height, (1.0,1.0,1.0), 0, self.options.use_pixel_space, 0.0);
                return
            }
        }
        self.add_texture(texture_path, "generic_name");
        println!("New texture created from the specified path. This message should only show up once for every new texture");
    }

    pub fn add_texture(&mut self, path: &str, name: &str) {
        for texture_batch in self.texture_batches.iter_mut() {
            if name.to_string() == texture_batch.path {
                return
            }
        }

        self.texture_batches.push(TextureBatch::new(&self.display, self.options.window_width, self.options.window_height, path.to_string()))
    }
}


#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
    style: i32,
    tex_coord: [f32; 2],
    variator: f32
}

implement_vertex!(Vertex, position, color, style, tex_coord, variator);


pub struct Batch {
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u16>,
    window_width: i32,
    window_height: i32,
    program: Program
}

impl Batch {
    fn new(display: &Display, window_width: i32, window_height: i32) -> Self {
        let fragment_shader = String::from("
            #version 140

            in vec4 v_color;
            in float v_style;
            in vec2 v_tex_coord;
            in float v_variator;

            out vec4 f_color;

            void main() {
              if (v_style == 1) {
                vec2 uv = v_tex_coord;
                
                float t = distance(uv, vec2(0.5));
                
                if (t <= 0.5) {
                  f_color = v_color;
                }
                else {
                  f_color = vec4(uv,1.0,0.0);
                }
              }
              else if (v_style == 2) {
                vec2 uv = v_tex_coord;
                
                if (uv.x + uv.y > 1 - v_variator) {
                  if (uv.x + uv.y < 1 + v_variator) {
                    f_color = v_color;
                  }
                  else {
                    f_color = vec4(uv,1.0,0.0);
                  }
                }
                
                else {
                  f_color = vec4(uv,1.0,0.0);
                }
              }
              else {
                f_color = v_color;
              }
            }
        ");

        let vertex_shader = String::from("
            #version 140

            in vec2 position;
            in vec4 color;
            in int style;
            in vec2 tex_coord;
            in float variator;

            out vec4 v_color;
            out float v_style;
            out vec2 v_tex_coord;
            out float v_variator;

            uniform mat4 matrix;

            void main() {
                v_color = color;
                v_style = style;
                v_tex_coord = tex_coord;
                v_variator = variator;
                gl_Position = matrix * vec4(position, 0.0, 1.0);
            }
        ");

        let program = { Program::from_source(display, &vertex_shader, &fragment_shader, None).unwrap() };

        Self { 
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
            window_width,
            window_height,
            program
        }
    }

    fn get_default_draw_params() -> DrawParameters<'static> {
        glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            polygon_mode: glium::PolygonMode::Fill,
            .. Default::default()
        }
    }

    fn draw(&mut self, frame: &mut Frame, display: &Display) {
        self.window_width = display.get_framebuffer_dimensions().0 as i32;
        self.window_height = display.get_framebuffer_dimensions().1 as i32;
        let uniforms = glium::uniform! {
            matrix: [
                [1.0, 0.0, 0.5, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0, 1.0, -1.0, 1.0f32],
            ]
        };

        let mut index_buffer_buffer: Vec<u16> = Vec::new();
        let mut vertex_buffer_buffer: Vec<Vertex> = Vec::new();
        let mut quad_count = 0;
        const QUADS_PER_DRAW: usize = 16380;

        for i in 0 .. self.vertex_buffer.len() {
            vertex_buffer_buffer.push(self.vertex_buffer[i]);
            
            if (i + 1) % 4 == 0 {
                for j in 0 .. 6 {
                    index_buffer_buffer.push(self.index_buffer[6 * quad_count + j]);
                }
                quad_count += 1;
            }
            if quad_count == QUADS_PER_DRAW || i == self.vertex_buffer.len() - 1 {
                let index_buffer = IndexBuffer::new(display, PrimitiveType::TrianglesList, &index_buffer_buffer).unwrap();
                let vertex_buffer = VertexBuffer::new(display, &vertex_buffer_buffer).unwrap();

                index_buffer_buffer = Vec::new();
                vertex_buffer_buffer = Vec::new();
                
                frame.draw(&vertex_buffer, &index_buffer, &self.program, &uniforms, &Batch::get_default_draw_params()).unwrap();

                quad_count = 0;
            }
        }

        self.vertex_buffer = Vec::new();
        self.index_buffer = Vec::new();
    }

    fn pixel_to_screenspace_invert_y_coord(&self, a: [f32; 2]) -> [f32; 2] {
        return [a[0] / ( self.window_width as f32 / 4.0 ), -a[1] / ( self.window_height as f32 / 4.0 )]
    }

    pub fn add_quad(&mut self, position: [i32; 2], width: i32, height: i32, color: (f32, f32, f32), style: i32, use_pixel_space: bool, variator: f32) {
        let mut position = Self::f32_2_array_to_i32_2(&position);
        if use_pixel_space {
            position = self.pixel_to_screenspace_invert_y_coord(position);
        }

        let width = width as f32 / ( self.window_width as f32 / 4.0 );
        let height = height as f32 / ( self.window_height as f32 / 4.0 );

        let index_buffer_size = self.vertex_buffer.len() as u16;
        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 1);
        self.index_buffer.push(index_buffer_size + 2);

        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 3);
        self.index_buffer.push(index_buffer_size + 2);

        let color = Self::color_tuple_to_array(&color);
        self.vertex_buffer.push(Vertex { position: [position[0]        , position[1]         ], color, style, tex_coord: [0.0,1.0], variator});
        self.vertex_buffer.push(Vertex { position: [position[0] + width, position[1]         ], color, style, tex_coord: [1.0,1.0], variator});
        self.vertex_buffer.push(Vertex { position: [position[0] + width, position[1] - height], color, style, tex_coord: [1.0, 0.0], variator});
        self.vertex_buffer.push(Vertex { position: [position[0]        , position[1] - height], color, style, tex_coord: [0.0,0.0], variator });
    }

    fn color_tuple_to_array(i: &(f32,f32,f32)) -> [f32; 4] {
        return [i.0,i.1,i.2,1.0]
    }
    fn f32_2_array_to_i32_2(i: &[i32; 2]) -> [f32; 2] {
        return [i[0] as f32, i[1] as f32]
    }

    pub fn raw_quad(&mut self, p1: [i32; 2], p2: [i32; 2], p3: [i32; 2], p4: [i32; 2], color: (f32, f32, f32), style: i32, use_pixel_space: bool, variator: f32) {
        let mut p1 = Self::f32_2_array_to_i32_2(&p1);
        let mut p2 = Self::f32_2_array_to_i32_2(&p2);
        let mut p3 = Self::f32_2_array_to_i32_2(&p3);
        let mut p4 = Self::f32_2_array_to_i32_2(&p4);

        if use_pixel_space {
            p1 = self.pixel_to_screenspace_invert_y_coord(p1);
            p2 = self.pixel_to_screenspace_invert_y_coord(p2);
            p3 = self.pixel_to_screenspace_invert_y_coord(p3);
            p4 = self.pixel_to_screenspace_invert_y_coord(p4);
        }

        let index_buffer_size = self.vertex_buffer.len() as u16;
        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 1);
        self.index_buffer.push(index_buffer_size + 2);

        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 3);
        self.index_buffer.push(index_buffer_size + 2);

        let color = Self::color_tuple_to_array(&color);
        self.vertex_buffer.push(Vertex { position: p1, color, style, tex_coord: [0.0,1.0], variator });
        self.vertex_buffer.push(Vertex { position: p2, color, style, tex_coord: [1.0,1.0], variator });
        self.vertex_buffer.push(Vertex { position: p3, color, style, tex_coord: [1.0,0.0], variator });
        self.vertex_buffer.push(Vertex { position: p4, color, style, tex_coord: [0.0,0.0], variator });
    }
}

pub struct TextureBatch {
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u16>,
    window_width: i32,
    window_height: i32,
    program: Program,
    path: String,
    texture: glium::texture::SrgbTexture2d
}

impl TextureBatch {
    fn new(display: &Display, window_width: i32, window_height: i32, path: String) -> Self {
        let vertex_shader = String::from("
            #version 140

            in vec2 position;
            in vec2 tex_coord;
            out vec2 v_tex_coord;

            uniform mat4 matrix;

            void main() {
                v_tex_coord = tex_coord;
                gl_Position = matrix * vec4(position, 0.0, 1.0);
            }
        ");

        let fragment_shader = String::from("       
            #version 140

            in vec2 v_tex_coord;
            out vec4 color;

            uniform sampler2D tex;

            void main() {
                color = texture(tex, v_tex_coord);
            }
        ");

        let program = Program::from_source(display, &vertex_shader, &fragment_shader, None).unwrap();

        let f = File::open(&path).unwrap();
        let mut reader = std::io::BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        let raw_texture: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::load(std::io::Cursor::new(&buffer),
                        image::ImageFormat::Png).unwrap().to_rgba8();
        let image_dimensions = raw_texture.dimensions();
        let image: glium::texture::RawImage2d<'_, u8> = glium::texture::RawImage2d::from_raw_rgba_reversed(&raw_texture.clone().into_raw(), image_dimensions);

        let texture: glium::texture::SrgbTexture2d = glium::texture::SrgbTexture2d::new(display, image).unwrap();

        Self { 
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
            window_width,
            window_height,
            program,
            path,
            texture
        }
    }

    fn draw(&mut self, frame: &mut Frame, display: &Display) {
        self.window_width = display.get_framebuffer_dimensions().0 as i32;
        self.window_height = display.get_framebuffer_dimensions().1 as i32;

        let uniforms = glium::uniform! {
            matrix: [
                [1.0, 0.0, 0.5, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0, 1.0, -1.0, 1.0f32],
            ],
            tex: &self.texture
        };

        let mut index_buffer_buffer: Vec<u16> = Vec::new();
        let mut vertex_buffer_buffer: Vec<Vertex> = Vec::new();
        let mut quad_count = 0;
        const QUADS_PER_DRAW: usize = 16380;

        for i in 0 .. self.vertex_buffer.len() {
            vertex_buffer_buffer.push(self.vertex_buffer[i]);
            
            if (i + 1) % 4 == 0 {
                for j in 0 .. 6 {
                    index_buffer_buffer.push(self.index_buffer[6 * quad_count + j]);
                }
                quad_count += 1;
            }
            if quad_count == QUADS_PER_DRAW || i == self.vertex_buffer.len() - 1 {
                let index_buffer = IndexBuffer::new(display, PrimitiveType::TrianglesList, &index_buffer_buffer).unwrap();
                let vertex_buffer = VertexBuffer::new(display, &vertex_buffer_buffer).unwrap();

                index_buffer_buffer = Vec::new();
                vertex_buffer_buffer = Vec::new();
                
                frame.draw(&vertex_buffer, &index_buffer, &self.program, &uniforms, &Batch::get_default_draw_params()).unwrap();

                quad_count = 0;
            }
        }

        self.vertex_buffer = Vec::new();
        self.index_buffer = Vec::new();
    }

    fn pixel_to_screenspace_invert_y_coord(&self, a: [f32; 2]) -> [f32; 2] {
        return [a[0] / ( self.window_width as f32 / 4.0 ), -a[1] / ( self.window_height as f32 / 4.0 )]
    }

    pub fn add_quad(&mut self, position: [i32; 2], width: i32, height: i32, color: (f32, f32, f32), style: i32, use_pixel_space: bool, variator: f32) {
        let mut position = Self::f32_2_array_to_i32_2(&position);
        if use_pixel_space {
            position = self.pixel_to_screenspace_invert_y_coord(position);
        }

        let width = width as f32 / ( self.window_width as f32 / 4.0 );
        let height = height as f32 / ( self.window_height as f32 / 4.0 );

        let index_buffer_size = self.vertex_buffer.len() as u16;
        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 1);
        self.index_buffer.push(index_buffer_size + 2);

        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 3);
        self.index_buffer.push(index_buffer_size + 2);

        let color = Self::color_tuple_to_array(&color);
        self.vertex_buffer.push(Vertex { position: [position[0]        , position[1]         ], color, style, tex_coord: [0.0,1.0], variator });
        self.vertex_buffer.push(Vertex { position: [position[0] + width, position[1]         ], color, style, tex_coord: [1.0,1.0], variator });
        self.vertex_buffer.push(Vertex { position: [position[0] + width, position[1] - height], color, style, tex_coord: [1.0, 0.0], variator });
        self.vertex_buffer.push(Vertex { position: [position[0]        , position[1] - height], color, style, tex_coord: [0.0,0.0], variator });
    }

    fn color_tuple_to_array(i: &(f32,f32,f32)) -> [f32; 4] {
        return [i.0,i.1,i.2,1.0]
    }
    fn f32_2_array_to_i32_2(i: &[i32; 2]) -> [f32; 2] {
        return [i[0] as f32, i[1] as f32]
    }

    pub fn raw_quad(&mut self, p1: [i32; 2], p2: [i32; 2], p3: [i32; 2], p4: [i32; 2], color: (f32, f32, f32), style: i32, use_pixel_space: bool, variator: f32) {
        let mut p1 = Self::f32_2_array_to_i32_2(&p1);
        let mut p2 = Self::f32_2_array_to_i32_2(&p2);
        let mut p3 = Self::f32_2_array_to_i32_2(&p3);
        let mut p4 = Self::f32_2_array_to_i32_2(&p4);

        if use_pixel_space {
            p1 = self.pixel_to_screenspace_invert_y_coord(p1);
            p2 = self.pixel_to_screenspace_invert_y_coord(p2);
            p3 = self.pixel_to_screenspace_invert_y_coord(p3);
            p4 = self.pixel_to_screenspace_invert_y_coord(p4);
        }

        let index_buffer_size = self.vertex_buffer.len() as u16;
        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 1);
        self.index_buffer.push(index_buffer_size + 2);

        self.index_buffer.push(index_buffer_size + 0);
        self.index_buffer.push(index_buffer_size + 3);
        self.index_buffer.push(index_buffer_size + 2);

        let color = Self::color_tuple_to_array(&color);
        self.vertex_buffer.push(Vertex { position: p1, color, style, tex_coord: [0.0,1.0], variator });
        self.vertex_buffer.push(Vertex { position: p2, color, style, tex_coord: [1.0,1.0], variator });
        self.vertex_buffer.push(Vertex { position: p3, color, style, tex_coord: [1.0,0.0], variator });
        self.vertex_buffer.push(Vertex { position: p4, color, style, tex_coord: [0.0,0.0], variator });
    }
}


pub fn new_texture(path: &str, display: &Display) -> glium::texture::SrgbTexture2d {
    let f = File::open(path).unwrap();
    let mut reader = std::io::BufReader::new(f);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();

    let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::load(std::io::Cursor::new(&buffer),
                    image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image: glium::texture::RawImage2d<'_, u8> = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let texture = glium::texture::SrgbTexture2d::new(display, image).unwrap();
    texture
}

pub struct Options {
    pub use_pixel_space: bool,
    pub window_width: i32,
    pub window_height: i32
}

impl Options {
    fn new(window_width: i32, window_height: i32) -> Self {
        Self {
            use_pixel_space: true,
            window_width,
            window_height
        }
    }
}
