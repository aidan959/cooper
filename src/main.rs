use application::application::CooperApplication;

fn main() {
    env_logger::init();
    let application = CooperApplication::create();
    application.run();

}