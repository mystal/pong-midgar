extern crate cgmath;
extern crate midgar;

mod app;

fn main() {
    let app_config = midgar::MidgarAppConfig::new()
        .with_title("pong")
        .with_screen_size((640, 400));
    let app: midgar::MidgarApp<app::GameApp> = midgar::MidgarApp::new(app_config);
    app.run();
}
