use std::{time::{Instant, Duration, self}, fs::File, io::Read};
pub use glium::glutin::event_loop::EventLoop;
pub use glium::glutin::event::VirtualKeyCode;
pub use glium::glutin::event::MouseButton;
use glium::{*, glutin::ContextBuilder};
use glutin::event::*;
use glutin::window::*;

enum Action {
    Continue
}

pub fn start_loop<F>(event_loop: EventLoop<()>, mut input_code: F)->! where F: 'static + FnMut(&Vec<Event<'_, ()>>) {
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
            next_frame_time = Instant::now() + Duration::from_nanos(1667);
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
                *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
            }
        };
    })
}


pub struct App {
    options: Options,
    display: Display,
    batch: Batch,
    texture_batches: Vec<TextureBatch>,
    last_frame_time: Instant,
    last_fps_output: Instant,
    animations: Vec<Animation>
}

impl App {
    pub fn new(title: &str, window_width: u32, window_height: u32) -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new().with_title(title).with_inner_size(glutin::dpi::Size::from(glutin::dpi::LogicalSize::new(window_width, window_height))).with_resizable(true);
        let context_buffer = ContextBuilder::new().with_depth_buffer(25);

        let display = Display::new(window_builder, context_buffer, &event_loop).unwrap();
        let batch = Batch::new(&display, window_width as i32, window_height as i32);

        ( App { display, batch, options: Options::new(window_width as i32, window_height as i32), texture_batches: Vec::new(), last_frame_time: Instant::now(), last_fps_output: Instant::now(), animations: vec![] }, event_loop)
    }
    // pub fn new_with_loop(title: &str, window_width: u32, window_height: u32, event_loop: EventLoop<()>) -> Self {
    //     let window_builder = WindowBuilder::new().with_title(title).with_inner_size(glutin::dpi::Size::from(glutin::dpi::LogicalSize::new(window_width, window_height))).with_resizable(true);
    //     let context_buffer = ContextBuilder::new().with_depth_buffer(25);

    //     let display = Display::new(window_builder, context_buffer, &event_loop).unwrap();
    //     let batch = Batch::new(&display, window_width as i32, window_height as i32);

    //     App { display, batch, options: Options::new(window_width as i32, window_height as i32), texture_batches: Vec::new(), last_frame_time: Instant::now(), last_fps_output: Instant::now() }
    // }

    pub fn use_pixel_coords(&mut self, param: bool) {
        self.options.use_pixel_space = param;
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

    pub fn animate(&mut self, position: [i32; 2], width: i32, height: i32, animation: &str) {
        let mut current_frame: &str = "";
        for i in self.animations.iter_mut() {
            if i.name == animation.to_string() {
                current_frame = i.textures[i.current_frame].as_str();
                if i.last_frame.elapsed() > Duration::from_secs_f32(1.0 / i.frame_rate as f32) {
                    i.last_frame = time::Instant::now();
                    if i.current_frame == i.textures.len() - 1 {
                        i.current_frame = 0;
                    }
                    else { i.current_frame += 1; }
                }
            }
        }
        blit(position, width, height, current_frame);
    }

    pub fn add_animation(&mut self, name: &str, frame_rate: u8, textures: Vec<&str>) {
        for ani in self.animations.iter() {
            if ani.name == name {
                return
            }
        }
        let textures = {
            let mut test: Vec<String> = vec![];
            for tex in textures {
                test.push(tex.to_string())
            }
            test
        };
        self.animations.push( Animation { name: name.to_string(), last_frame: time::Instant::now(), textures, frame_rate, current_frame: 0 })
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
                let index_buffer = IndexBuffer::new(display, index::PrimitiveType::TrianglesList, &index_buffer_buffer).unwrap();
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

        let behavior = glium::uniforms::SamplerBehavior {
            minify_filter: uniforms::MinifySamplerFilter::Linear,
            magnify_filter: uniforms::MagnifySamplerFilter::Nearest,
            ..Default::default()
        };

        let uniforms = glium::uniform! {
            matrix: [
                [1.0, 0.0, 0.5, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-1.0, 1.0, -1.0, 1.0f32],
            ],
            tex: glium::uniforms::Sampler(&self.texture, behavior)
        };

        let mut index_buffer_buffer: Vec<u16> = Vec::new();
        let mut vertex_buffer_buffer: Vec<Vertex> = Vec::new();
        let mut quad_count: usize = 0;
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
                let index_buffer: IndexBuffer<u16> = IndexBuffer::new(display, index::PrimitiveType::TrianglesList, &index_buffer_buffer).unwrap();
                let vertex_buffer: VertexBuffer<Vertex> = VertexBuffer::new(display, &vertex_buffer_buffer).unwrap();

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

static mut CONTEXT: Option<App> = None;
static mut PRESSED_KEYS: Vec<VirtualKeyCode> = Vec::new();
static mut PRESSED_BUTTONS: Vec<MouseButton> = Vec::new();
static mut CURSOR_POSITION: [f32; 2] = [0.0,0.0];

pub fn initialize(title: &str, width: u32, height: u32) -> EventLoop<()> {
    unsafe {
        let (app, event_loop) = App::new(title, width, height);
        CONTEXT = Some ( app );
        return event_loop
    };
}

pub fn get_app() -> &'static mut App {
    if unsafe { CONTEXT.is_none() } { panic!() }
    return unsafe { CONTEXT.as_mut().unwrap() }
}

pub fn circle(position: [i32; 2], radius: i32, color: (f32,f32,f32)) {
    let app = get_app();
    app.circle([position[0] as i32, position[1] as i32], radius as i32, color)
}
pub fn square(position: [i32; 2], size: i32, color: (f32,f32,f32)) {
    let app = get_app();
    app.square([position[0] as i32, position[1] as i32], size as i32, color)
}
pub fn rect(position: [i32; 2], width: i32, height: i32, color: (f32,f32,f32)) {
    let app = get_app();
    app.rect([position[0] as i32, position[1] as i32], width as i32, height as i32, color)
}
pub fn blit(position: [i32; 2], width: i32, height: i32, texture_path: &str) {
    let app = get_app();
    app.texture_quad([position[0] as i32, position[1] as i32], width as i32, height as i32, texture_path)
}
pub fn animate(position: [i32; 2], width: i32, height: i32, animation: &str) {
    let app = get_app();
    app.animate(position, width, height, animation);
}
pub fn add_animation(name: &str, frame_rate: u8, textures: Vec<&str>) {
    let app = get_app();
    app.add_animation(name, frame_rate, textures);
}
struct Animation {
    name: String,
    last_frame: time::Instant,
    textures: Vec<String>,
    frame_rate: u8,
    current_frame: usize
}

pub fn run<F>(title: &str, window_width: u32, window_height: u32, mut loop_function: F) ->! where F: 'static + FnMut() {
    let ev_loop = initialize(title, window_width, window_height);
    let app = get_app();
    start_loop(ev_loop, move | events | {
        loop_function();

        app.finish([0.1,0.1,0.1], events)
    });
}

pub fn start_loop_and_init<F>(title: &str, width: u32, height: u32, mut input_code: F)->! where F: 'static + FnMut(&Vec<Event<'_, ()>>) {
    let ev_loop = initialize(title, width, height);
    let app = get_app();
    start_loop(ev_loop, move | events | {
        input_code(events);

        for event in events.iter() {
            match event {
                Event::WindowEvent { event, ..} => match event {
                    glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                        if input.virtual_keycode.is_some() {
                            unsafe {
                                if PRESSED_KEYS.contains(&input.virtual_keycode.unwrap()) {
                                    PRESSED_KEYS.remove(PRESSED_KEYS.iter().position(|r| r == &input.virtual_keycode.unwrap()).unwrap());
                                }
                                else {
                                    PRESSED_KEYS.push(input.virtual_keycode.unwrap());
                                }
                            }
                        }
                    },
                    glutin::event::WindowEvent::MouseInput {button, .. } => {
                        unsafe {
                            if PRESSED_BUTTONS.contains(button) {
                                PRESSED_BUTTONS.remove(PRESSED_BUTTONS.iter().position(|r| r == button).unwrap());
                            }
                            else {
                                PRESSED_BUTTONS.push(*button);
                            }
                        }
                    },
                    glutin::event::WindowEvent::CursorMoved { position, .. } => {
                        unsafe { CURSOR_POSITION = [position.x as f32, position.y as f32] }
                    }
                    _ => ()
                },
                _ => ()
            }
        }

        app.finish([0.1,0.1,0.1], events)
    });
}


pub fn start<F>(event_loop: EventLoop<()>, mut loop_function: F) ->! where F: 'static + FnMut(&Vec<Event<'_, ()>>) {
    let app = get_app();
    start_loop(event_loop, move | events | {
        loop_function(events);

        for event in events.iter() {
            match event {
                Event::WindowEvent { event, ..} => match event {
                    glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                        if input.virtual_keycode.is_some() {
                            unsafe {
                                if PRESSED_KEYS.contains(&input.virtual_keycode.unwrap()) {
                                    PRESSED_KEYS.remove(PRESSED_KEYS.iter().position(|r| r == &input.virtual_keycode.unwrap()).unwrap());
                                }
                                else {
                                    PRESSED_KEYS.push(input.virtual_keycode.unwrap());
                                }
                            }
                        }
                    },
                    glutin::event::WindowEvent::MouseInput {button, .. } => {
                        unsafe {
                            if PRESSED_BUTTONS.contains(button) {
                                PRESSED_BUTTONS.remove(PRESSED_BUTTONS.iter().position(|r| r == button).unwrap());
                            }
                            else {
                                PRESSED_BUTTONS.push(*button);
                            }
                        }
                    }
                    _ => ()
                },
                _ => ()
            }
        }

        app.finish([0.1,0.1,0.1], events)
    });
}

pub fn key_pressed(key: VirtualKeyCode) -> bool {
    return unsafe { PRESSED_KEYS.contains(&key) }
}

pub fn keys_pressed() -> Vec<VirtualKeyCode> {
    return unsafe { PRESSED_KEYS.clone() }
}

static mut PRESSED_TOGGLE: bool = false;
pub fn key_press(button: VirtualKeyCode) -> bool {
    unsafe {
        if PRESSED_TOGGLE == true {
            if !PRESSED_KEYS.contains(&button) { PRESSED_TOGGLE = false }
            return false
        }
        else {
            if PRESSED_KEYS.contains(&button) { PRESSED_TOGGLE = true; return true }
            return false
        }
    }
}
static mut KEY_RELEASED_TOGGLE: bool = true;
pub fn key_release(button: VirtualKeyCode) -> bool {
    unsafe {
        if KEY_RELEASED_TOGGLE == true {
            if !PRESSED_KEYS.contains(&button) { KEY_RELEASED_TOGGLE = false }
            return false
        }
        else {
            if PRESSED_KEYS.contains(&button) { KEY_RELEASED_TOGGLE = true; return true }
            return false
        }
    }
}

pub fn mouse_pos() -> [f32; 2] {
    unsafe { CURSOR_POSITION.clone() }
}

pub fn mouse_clicks() -> Vec<MouseButton> {
    return unsafe { PRESSED_BUTTONS.clone() }
}

static mut CLICKED_TOGGLE: bool = false;
pub fn mouse_clicked(button: MouseButton) -> bool {
    unsafe {
        if CLICKED_TOGGLE == true {
            if !PRESSED_BUTTONS.contains(&button) { CLICKED_TOGGLE = false }
            return false
        }
        else {
            if PRESSED_BUTTONS.contains(&button) { CLICKED_TOGGLE = true; return true }
            return false
        }
    }
}

static mut RELEASED_TOGGLE: bool = true;
pub fn mouse_released(button: MouseButton) -> bool {
    unsafe {
        if RELEASED_TOGGLE == true {
            if PRESSED_BUTTONS.contains(&button) { RELEASED_TOGGLE = false }
            return false
        }
        else {
            if !PRESSED_BUTTONS.contains(&button) { RELEASED_TOGGLE = true; return true }
            return false
        }
    }
}

pub fn get_dims() -> [u32; 2] {
    let app = get_app();
    return [app.options.window_width as u32, app.options.window_height as u32]
}

pub fn text(string: &str, pos: [i32; 2], size: i32) {
    for (i, char) in string.to_uppercase().chars().enumerate() {
        match char {
            'B' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/b.png"),
            'A' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/a.png"),
            'C' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/c.png"),
            'D' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/d.png"),
            'E' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/e.png"),
            'F' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/f.png"),
            'G' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/g.png"),
            'H' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/h.png"),
            'I' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/i.png"),
            'J' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/j.png"),
            'K' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/k.png"),
            'L' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/l.png"),
            'M' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/m.png"),
            'N' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/n.png"),
            'O' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/o.png"),
            'P' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/p.png"),
            'Q' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/q.png"),
            'R' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/r.png"),
            'S' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/s.png"),
            'T' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/t.png"),
            'U' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/u.png"),
            'V' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/v.png"),
            'W' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/w.png"),
            'X' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/x.png"),
            'Y' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/y.png"),
            'Z' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/z.png"),
            '!' => blit([pos[0] + size * i as i32, pos[1]], size, size, "src/assets/font/!.png"),
            _ => ()
        }
    }
}