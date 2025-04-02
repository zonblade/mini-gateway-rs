use mini_config::Configure;

#[derive(Debug, Clone, Configure)]
pub enum Api {
    TCPAddress
}

pub fn init(){
    Api::TCPAddress.set("127.0.0.1:30099");
}