mod file_dialog;
mod message_dialog;

mod modal_future;

mod policy_manager;

mod utils;

use cocoa_foundation::base::id;

pub(self) trait AsModal {
    fn modal_ptr(&self) -> id;
}
