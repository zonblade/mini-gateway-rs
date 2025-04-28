use crate::module::{udp_log_fetcher, udp_logger};


pub fn init(){
    let proxy_consumer = udp_logger::get_proxy_log_consumer();

    // // Optional: Spawn threads to process messages from each consumer
    if let Some(consumer) = proxy_consumer {
        std::thread::spawn(move || {
            log::info!("Started proxy log consumer thread");
            loop {
                match consumer.recv() {
                    Ok(msg) => {

                        let scoped_format = {
                            let splitted = msg.message.clone()
                                .split(',')
                                .into_iter() // Take until we hit an empty section
                                .map(|s| {
                                    let data = s
                                        .trim()
                                        .split(':')
                                        .into_iter()
                                        .map(|s| s.trim().to_string())
                                        .collect::<Vec<String>>();
                                    (data[0].clone(), data[1].clone())
                                })
                                .collect::<Vec<(String, String)>>();

                            udp_log_fetcher::LogMessageFormatted {
                                id: splitted
                                    .iter()
                                    .find(|(k, _)| k == "ID")
                                    .map_or("".to_string(), |(_, v)| v.clone()),
                                connection_type: splitted
                                    .iter()
                                    .find(|(k, _)| k == "CONN")
                                    .map_or("".to_string(), |(_, v)| v.clone()),
                                status: splitted
                                    .iter()
                                    .find(|(k, _)| k == "STATUS")
                                    .map_or("".to_string(), |(_, v)| v.clone()),
                                packet_size: splitted
                                    .iter()
                                    .find(|(k, _)| k == "SIZE")
                                    .map_or(0, |(_, v)| v.parse().unwrap_or(0)),
                                comment: splitted
                                    .iter()
                                    .find(|(k, _)| k == "COMMENT")
                                    .map_or("".to_string(), |(_, v)| v.clone()),
                            }
                        };
                        log::info!("{:?}", scoped_format)
                    },
                    Err(_) => break,
                }
            }
        });
    }
}