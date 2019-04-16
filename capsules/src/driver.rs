use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
// syscall driver numbers
pub enum NUM {
    ADC = 0x00000005,
    ALARM = 0x00000000,
    AMBIENT_LIGHT = 0x60002,
    ANALOG_COMPARATOR = 0x00007,
    APP_FLASH =  0x50000,
    BATTERY = 0x0000000B,
    BLE_ADVERTISING = 0x030000,
    BUTTON = 0x00000003,
    CONSOLE = 0x0000ABCD,
    UART = 0x00000001,
    CRC = 0x40002,
    DAC = 0x00000006,
    GPIO = 0x00000004,
    GPIO_ASYNC = 0x80003,
    GPS = 0x80005,
    HUMIDITY = 0x60001,
    I2C_MASTER = 0x40006,
    I2C_MASTER_SLAVE = 0x20006,
    LED = 0x2,
    LPS25HB = 0x70004,
    LTC294X = 0x80000,
    MAX17205 = 0x80001,
    NINEDOF = 0x60004,
    NVM_STORAGE = 0x50001,
    NRF51822_SERIALIZATION = 0x80004,
    PCA9544A = 0x80002,
    RNG = 0x40001,
    SD_CARD = 0x50002,
    SKY2435L = 0x484c4d03, // ascii hex val for "HLMx"
    SPI = 0x20001,
    TEMPERATURE = 0x60000,
    TMP006 = 0x70001,
    TSL2561 = 0x70000,
    USB_USER = 0x20005,
}
}
