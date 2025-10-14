pub mod account;
pub mod directory;
pub mod switch;
pub mod webdav;
pub mod logs;
pub mod base_url;
pub mod settings;

use comfy_table::{Table, presets::UTF8_FULL};

pub fn create_table() -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table
}
