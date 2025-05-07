use super::listen_proxy;

pub fn spawn_all(){
    // spawn on thread
    std::thread::spawn(move || {
        listen_proxy();
    });
}