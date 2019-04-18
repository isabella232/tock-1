//! Interface for reading battery value

pub trait Reader {
    /// Initiate a CRC calculation
    fn get_mv(&self) -> u32;
}

pub trait Client {
    /// Receive the successful result of a CRC calculation
    fn receive_result(&self, _: u32);
}
