use blithaven::*;

fn main() {
    let (mut app, event_loop) = App::new("t", 300, 500);

    blithaven::start_loop(event_loop, move | events | {
        app.finish([0.1,0.1,0.1], events)
    });
    run(func);
}


fn func() {
    circle([0.0,0.0], 10.0, (1.0,1.0,1.0));
    other_func();
}

fn other_func() {
    circle([300.0,120.0], 13.5, (0.4,0.6,0.1));
}