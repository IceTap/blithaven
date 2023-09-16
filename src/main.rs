use blithaven::*;

fn main() {
    start_loop_and_init("super awesome window", 500, 300, move | events | {
        func();
        // let keys = mouse_clicks(events);
        // if keys.len() > 0 { println!("{:?}", keys) }
        if mouse_clicked(MouseButton::Right, events) {
            println!("CLicked")
        }
    })
}

fn func() {
    circle([0.0,0.0], 10.0, (1.0,1.0,1.0));
    other_func();
}

fn other_func() {
    circle([300.0,120.0], 13.5, (0.4,0.6,0.1));
    texture([100.0,100.0], 30.0, 30.0, "./src/assets/Slime_32x32.png")
}