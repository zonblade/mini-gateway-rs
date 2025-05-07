use crate::module::udp_net::{udp_log_db::init_with_table, udp_log_processor::UdpLogProcessor, udp_logger};

pub fn init() {
    let gateway_consumer = udp_logger::get_gateway_log_consumer();

    // // Optional: Spawn threads to process messages from each consumer
    if let Some(consumer) = gateway_consumer {
        std::thread::spawn(move || {
            log::info!("Started proxy log consumer thread");
            let db = init_with_table("log_gateway");
            let _ = db.init_database();
            let _ = db.start();
            let logs = UdpLogProcessor::new(consumer, db);
            logs.start_processing_thread();
        });
    }
}
