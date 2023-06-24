use blithaven;

fn main() {
    let (mut app, event_loop) = blithaven::App::init_with_loop("test");

    let mut ball_x = 0.0;
    let mut ball_y = 0.0;
    let mut ball_vel_x = 0.02;
    let mut ball_vel_y = 0.01;

    let mut paddle_1_pos = 0.7;
    let mut paddle_2_pos = 0.0;

    const PADDLEHEIGHT: f32 = 0.4;
    const BALLRADIUS: f32 = 0.05;

    blithaven::run(event_loop, move | events | {
        
        ball_x += ball_vel_x;
        ball_y += ball_vel_y;

        if ( ball_x + BALLRADIUS >= 1.2 && ( ball_y < paddle_1_pos + PADDLEHEIGHT / 2.0 && ball_y > paddle_1_pos - PADDLEHEIGHT / 2.0 ) ) || ball_x - BALLRADIUS <= -1.2 && ( ball_y < paddle_2_pos + PADDLEHEIGHT / 2.0 && ball_y > paddle_2_pos - PADDLEHEIGHT / 2.0 ){
            ball_vel_x *= -1.0;
        }
        if ball_y + BALLRADIUS >= 1.0 || ball_y - BALLRADIUS <= -1.0 {
            ball_vel_y *= -1.0;
        }
 
        if blithaven::key_pressed(events) == "W" {
            paddle_2_pos += 0.1;
        }
        if blithaven::key_pressed(events) == "S" {
            paddle_2_pos -= 0.1;
        }
        if blithaven::key_pressed(events) == "Up" {
            paddle_1_pos += 0.1;
        }
        if blithaven::key_pressed(events) == "Down" {
            paddle_1_pos -= 0.1;
        }

        app.circle([ball_x,ball_y], BALLRADIUS, (1.0,1.0,1.0));
        app.quad([1.2, paddle_1_pos + PADDLEHEIGHT / 2.0], 0.05, PADDLEHEIGHT, (1.0,1.0,1.0));
        app.quad([-1.25, paddle_2_pos + PADDLEHEIGHT / 2.0], 0.05, PADDLEHEIGHT, (1.0,1.0,1.0));

        app.save_frame((0.01,0.01,0.01), events);
    });
}
