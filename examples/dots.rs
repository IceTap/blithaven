use blithaven;

fn main() {

    let (mut app, event_loop) = blithaven::App::init_with_loop("dots");

    let mut zoom = 1.0;
    let mut camera_position = [0.0,0.0];

    blithaven::run(event_loop, move | events | {
        
        for i in -100 .. 100 {
            for j in -100 .. 100 {
                app.circle([i as f32 * 0.3, j as f32 * 0.3], 0.05, (0.1,0.1,0.1));
            }
        }

        app.circle([0.4,0.0], 0.5, (1.0,0.2,0.2));
        app.square([-0.9,0.9], 0.4, (0.3,1.0,0.3));

        blithaven::arrow_key_zoom(events, &mut app, &mut zoom);

        if blithaven::key_pressed(events) == "W" {
            camera_position[1] -= 0.1;
        }
        if blithaven::key_pressed(events) == "A" {
            camera_position[0] += 0.1;
        }
        if blithaven::key_pressed(events) == "S" {
            camera_position[1] += 0.1;
        }
        if blithaven::key_pressed(events) == "D" {
            camera_position[0] -= 0.1;
        }

        app.set_positional_offset(camera_position);
        
        app.save_frame((0.01,0.01,0.01), events)
    });
}
