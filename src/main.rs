use blithaven;
use rand::Rng;

fn main() {
    let (mut app, mut event_loop) = blithaven::App::new("title", 800, 800);
    let mut rng = rand::thread_rng();

    let mut balls: Vec<Ball> = Vec::new();
    balls.push(Ball::new([200.0,200.0], [0.1,0.1], (1.0,1.0,1.0), 10.0));
    balls.push(Ball::new([200.0,250.0], [0.1,-0.1], (1.0,1.0,1.0), 10.0));

    for i in 0 .. 10 {
        balls.push(Ball::new([rng.gen::<f32>() * 300.0 + 10.0, rng.gen::<f32>() * 300.0 + 10.0], [rng.gen::<f32>() - 0.5, rng.gen::<f32>() - 0.5], (rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()), 10.0));
    }

    blithaven::start(event_loop, move | events | {
        
        let mut buffer_balls = {
            let mut buffer_balls: Vec<Ball> = Vec::new();
            for i in 0 .. balls.len() {
                buffer_balls.push(balls[i])
            }
            buffer_balls
        };

        for i in 0 .. balls.len() {
            let mut current_ball = balls[i];
            current_ball.apply_vel();
            current_ball.draw(&mut app);
            current_ball.bounce_walls();

            for (index, other_ball) in balls.iter_mut().enumerate() {
                if i == index { continue }

                current_ball.collide(other_ball);
            }

            buffer_balls[i] = current_ball
        }
        balls = buffer_balls;


        app.finish([0.05,0.05,0.05], events)
    })
}


#[derive(Copy, Clone)]
struct Ball {
    pos: Vector,
    vel: Vector,
    color: (f32, f32, f32),
    radius: f32
}

impl Ball {
    fn new(pos: [f32; 2], vel: [f32; 2], color: (f32,f32,f32), radius: f32) -> Self {
        let pos = Vector::new(pos);
        let vel = Vector::new(vel);
        Self {
            pos,
            vel,
            color,
            radius
        }
    }
    fn apply_vel(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
    }
    fn bounce_walls(&mut self) {
        if self.pos.new_add(&self.vel).y + self.radius > 400.0 || self.pos.new_add(&self.vel).y - self.radius < 0.0 {
            self.vel.y *= -1.0;
        }
        if self.pos.new_add(&self.vel).x + self.radius > 400.0 || self.pos.new_add(&self.vel).x - self.radius < 0.0 {
            self.vel.x *= -1.0;
        }
    }
    fn draw(&self, app: &mut blithaven::App) {
        app.circle([self.pos.x as i32, self.pos.y as i32], self.radius as i32, self.color);
    }
    fn collide(&mut self, other: &Ball) {
        if self.pos.positional_dist(&other.pos) < self.radius + other.radius {
            self.color = (1.0,0.0,0.0);

            let mut direction_from_other_node: Vector = other.vel.new_sub(&self.vel);

            self.vel.add(&direction_from_other_node);
            direction_from_other_node.set_mag((self.radius + other.radius) - self.pos.positional_dist(&other.pos));
            self.pos.add(&direction_from_other_node);
            self.pos.add(&self.vel);

            Vector::ceil(&mut self.vel, 0.2);
        }
        else {
            self.color = (1.0,1.0,1.0);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Vector {
    x: f32,
    y: f32
}

impl Vector {
    fn new(val: [f32; 2]) -> Self {
        Self { x: val[0], y: val[1] }
    }

    fn positional_dist(&self, other: &Vector) -> f32 {
        return f32::sqrt(f32::powi(other.x - self.x, 2) + f32::powi(other.y - self.y, 2))
    }

    fn mag(&self) -> f32 {
        return f32::sqrt((self.x * self.x) + (self.y * self.y))
    }

    fn add(&mut self, other: &Vector) {
        self.x += other.x;
        self.y += other.y;
    }
    fn sub(&mut self, other: &Vector) {
        self.x -= other.x;
        self.y -= other.y;
    }
    fn set_mag(&mut self, i: f32) {
        let past_mag = self.mag();
        self.x *= i / past_mag;
        self.y *= i / past_mag;
    }
    fn new_add(&self, other: &Vector) -> Vector {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }
    fn new_sub(&self, other: &Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }

    fn ceil(vec: &mut Vector, max: f32) {
        if vec.mag() > max { vec.set_mag(max) }
    }
}
