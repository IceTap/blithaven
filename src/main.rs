use blithaven_rewrite;

fn main() {
    let (mut app, mut event_loop) = blithaven_rewrite::App::new("title", 800, 800);

    blithaven_rewrite::start(event_loop, move | events | {
        app.circle([100,100], 10, (0.5,0.6,0.3));
        app.line([100,100], [120,200], 10, (1.0,1.0,1.0));

        app.texture_quad([300,200], 100, 100, "src/Slime_32x32.png");
        app.texture_quad([50,200], 100, 100, "src/dirt_8x8.png");

        app.finish([0.05,0.05,0.05], events)
    })
}
