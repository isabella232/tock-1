use enum_primitive::cast::FromPrimitive;

enum_from_primitive!{
pub enum PIN_FN {
    UART0_RX = 12,
    UART0_TX = 13,
    I2C0_SCL = 22,
    I2C0_SDA = 5,
    TDO = 16,
    TDI = 17,
    RED_LED = 6,
    GREEN_LED = 7,
    BUTTON_1 = 15,
    BUTTON_2 = 14,
    GPIO0 = 20,
    ADC0 = 30,
    ADC1 = 29,
    ADC2 = 28,
    ADC3 = 27,
    ADC4 = 26,
    ADC5 = 25,
    ADC6 = 24,
    ADC7 = 23,
}
}
