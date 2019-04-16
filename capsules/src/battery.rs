use crate::enum_primitive::cast::FromPrimitive;

use kernel::hil;
use kernel::{AppId, Driver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::BATTERY as usize;

pub struct Battery<'a, G: hil::battery::Reader> {
    reader: &'a G,
}


impl<'a, G: hil::battery::Reader> Battery<'a, G> {
    pub fn new(
        reader: &'a G,
    ) -> Battery <'a, G> {

        Battery {
            reader: reader,
        }
    }

}

enum_from_primitive! {
#[derive(Debug, Clone, Copy)]
pub enum Command {
    GetValue = 1
}
}
  
impl<'a, G: hil::battery::Reader> Driver for Battery<'a, G> {
    fn command(&self, command_num: usize, _data: usize, _: usize, _appid: AppId) -> ReturnCode {

        if let Some(num) = Command::from_usize(command_num) {
            match num {
                Command::GetValue => {
                    ReturnCode::SuccessWithValue {
                        value: self.reader.get_mv() as usize,
                    }
                    
                }
            }
        } else {
            ReturnCode::ENOSUPPORT
        }
    }
}
