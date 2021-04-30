fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    //run::<GameData, Game<IsometricCamera>>(window, std::path::Path::new("../content"));
}