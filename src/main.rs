use blithaven;

fn main() {
    let (mut app, event_loop) = blithaven::App::init_with_loop("test");

    blithaven::run(event_loop, move | events | {
        app.circle([0.0,0.0], 0.1, (1.0,1.0,1.0));

        app.save_frame((0.01,0.01,0.01), events);
    });
}
