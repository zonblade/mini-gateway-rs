pub mod home;
pub mod users;
pub mod settings;

pub use home::home as home_handler;
pub use users::users as users_handler;
pub use settings::{proxy_handler, gwnode_handler, gateway_handler};