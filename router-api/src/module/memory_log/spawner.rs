use super::listen_proxy;

pub fn spawn_all(){
    // spawn on thread
    std::thread::spawn(move || {
        log::info!("Starting memory log spawner...");
        listen_proxy();
    });
}