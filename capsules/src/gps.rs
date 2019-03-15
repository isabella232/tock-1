use kernel::common::cells::{MapCell, OptionalCell, TakeCell};

use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::GPS as usize;

