use crate::module::udp_logger;

pub fn init() {
    let normal_consumer = udp_logger::get_normal_log_consumer();

    // // Optional: Spawn threads to process messages from each consumer
    if let Some(consumer) = normal_consumer {
        std::thread::spawn(move || {
            log::info!("Started proxy log consumer thread");
            loop {
                match consumer.recv() {
                    Ok(msg) => {
                        log::info!("{:?}", msg.message)
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
