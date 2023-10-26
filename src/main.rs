use application::application::CooperApplication;


fn main() {
    env_logger::init();
    let appplication = CooperApplication::create();
    appplication.run();
}
