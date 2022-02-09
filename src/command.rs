use crate::server::FormData;

pub enum Command {
    SaveConfig(FormData),
    SwitchPage,
}
