use crate::returncode::ReturnCode;

pub trait SE2435L {
    fn sleep(&self) -> ReturnCode;
    fn bypass(&self) -> ReturnCode;
    fn enable_pa(&self) -> ReturnCode;
    fn enable_lna(&self) -> ReturnCode;
}
