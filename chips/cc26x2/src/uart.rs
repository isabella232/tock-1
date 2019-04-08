//! UART driver, cc26x2 family
use crate::chip::SleepMode;
use crate::peripheral_interrupts;
use crate::peripheral_manager::PowerClient;
use crate::power::PM;
use crate::prcm;
use core::cell::Cell;
use cortexm4::nvic;
use kernel;
use kernel::common::cells::{MapCell, OptionalCell};
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::uart;
use kernel::ReturnCode;

use crate::memory_map::{UART0_BASE, UART1_BASE};

const MCU_CLOCK: u32 = 48_000_000;

#[repr(C)]
struct UartRegisters {
    dr: ReadWrite<u32>,
    rsr_ecr: ReadWrite<u32>,
    _reserved0: [u32; 0x4],
    fr: ReadOnly<u32, Flags::Register>,
    _reserved1: [u32; 0x2],
    ibrd: ReadWrite<u32, IntDivisor::Register>,
    fbrd: ReadWrite<u32, FracDivisor::Register>,
    lcrh: ReadWrite<u32, LineControl::Register>,
    ctl: ReadWrite<u32, Control::Register>,
    ifls: ReadWrite<u32, FifoLevelSelect::Register>,
    imsc: ReadWrite<u32, Interrupts::Register>,
    ris: ReadOnly<u32, Interrupts::Register>,
    mis: ReadOnly<u32, Interrupts::Register>,
    icr: WriteOnly<u32, Interrupts::Register>,
    dmactl: ReadWrite<u32>,
}

pub static mut UART0: UART = UART::new(&UART0_REG, &UART0_NVIC, 0);
pub static mut UART1: UART = UART::new(&UART1_REG, &UART1_NVIC, 1);

register_bitfields![
    u32,
    Control [
        UART_ENABLE OFFSET(0) NUMBITS(1) [],
        LB_ENABLE OFFSET(7) NUMBITS(1) [],
        TX_ENABLE OFFSET(8) NUMBITS(1) [],
        RX_ENABLE OFFSET(9) NUMBITS(1) []
    ],
    LineControl [
        FIFO_ENABLE OFFSET(4) NUMBITS(1) [],
        WORD_LENGTH OFFSET(5) NUMBITS(2) [
            Len5 = 0x0,
            Len6 = 0x1,
            Len7 = 0x2,
            Len8 = 0x3
        ]
    ],
    FifoLevelSelect [
        RXSEL OFFSET(3) NUMBITS(3) [
            OneEighth = 0,
            OneQuarter = 1,
            Half = 2,
            ThreeQuarters = 3,
            SevenEights = 4
        ],
        TXSEL OFFSET(0) NUMBITS(3) [
            OneEighth = 0,
            OneQuarter = 1,
            Half = 2,
            ThreeQuarters = 3,
            SevenEights = 4
        ]
    ],
    IntDivisor [
        DIVISOR OFFSET(0) NUMBITS(16) []
    ],
    FracDivisor [
        DIVISOR OFFSET(0) NUMBITS(6) []
    ],
    Flags [
        CTS OFFSET(0) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
        RX_FIFO_EMPTY OFFSET(4) NUMBITS(1) [],
        TX_FIFO_FULL OFFSET(5) NUMBITS(1) [],
        RX_FIFO_FULL OFFSET(6) NUMBITS(1) [],
        TX_FIFO_EMPTY OFFSET(7) NUMBITS(1) []
    ],
    Interrupts [
         ALL_INTERRUPTS OFFSET(0) NUMBITS(12) [
            // sets all interrupts without writing 1's to reg with undefined behavior
            Set =  0b111111110010,
            // you are allowed to write 0 to everyone
            Clear = 0x000000
        ],
        CTSIMM OFFSET(1) NUMBITS(1) [],              // clear to send interrupt mask
        RX OFFSET(4) NUMBITS(1) [],                  // receive interrupt mask
        TX OFFSET(5) NUMBITS(1) [],                  // transmit interrupt mask
        RX_TIMEOUT OFFSET(6) NUMBITS(1) [],          // receive timeout interrupt mask
        FE OFFSET(7) NUMBITS(1) [],                  // framing error interrupt mask
        PE OFFSET(8) NUMBITS(1) [],                  // parity error interrupt mask
        BE OFFSET(9) NUMBITS(1) [],                  // break error interrupt mask
        OE OFFSET(10) NUMBITS(1) [],                 // overrun error interrupt mask
        END_OF_TRANSMISSION OFFSET(11) NUMBITS(1) [] // end of transmission interrupt mask
    ]
];

const UART0_REG: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(UART0_BASE as *const UartRegisters) };

const UART1_REG: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(UART1_BASE as *const UartRegisters) };

const UART0_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::UART0 as u32) };
const UART1_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::UART1 as u32) };

/// Stores an ongoing TX transaction
struct Transaction {
    /// The buffer containing the bytes to transmit as it should be returned to
    /// the client
    buffer: &'static mut [u8],
    /// The total amount to transmit
    length: usize,
    /// The index of the byte currently being sent
    index: usize,
    newline: bool,
}
#[allow(dead_code)]
pub struct UART<'a> {
    registers: &'static StaticRef<UartRegisters>,
    nvic: &'static nvic::Nvic,
    num: usize,
    tx_client: OptionalCell<&'a uart::TransmitClient>,
    rx_client: OptionalCell<&'a uart::ReceiveClient>,
    tx: MapCell<Transaction>,
    rx: MapCell<Transaction>,
    receiving_word: Cell<bool>,
}

macro_rules! uart_nvic {
    ($fn_name:tt, $uart:ident) => {
        #[inline(never)]
        pub extern "C" fn $fn_name() {
            unsafe {
                // handle RX
                $uart.rx.map(|rx| {
                    while $uart.rx_fifo_not_empty() && rx.index < rx.length && !rx.newline {
                        let byte = $uart.read_byte();
                        rx.buffer[rx.index] = byte;
                        rx.index += 1;
                        if byte == b'\n' {
                            rx.newline = true;
                            rx.buffer[rx.index] = b'\0';
                            rx.index += 1;
                        }
                    }
                });

                $uart.tx.map(|tx| {
                    // if a big buffer was given, this could be a very long call
                    if $uart.tx_fifo_not_full() && tx.index < tx.length {
                        $uart.send_byte(tx.buffer[tx.index]);
                        tx.index += 1;
                    }
                });
                $uart.registers.icr.write(Interrupts::ALL_INTERRUPTS::Set);
                $uart.nvic.clear_pending();
            }
        }
    };
}

uart_nvic!(uart0_isr, UART0);
uart_nvic!(uart1_isr, UART1);

impl<'a> UART<'a> {
    const fn new(
        registers: &'static StaticRef<UartRegisters>,
        nvic: &'static nvic::Nvic,
        num: usize,
    ) -> UART<'a> {
        UART {
            registers,
            nvic,
            num,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx: MapCell::empty(),
            rx: MapCell::empty(),
            receiving_word: Cell::new(false),
        }
    }

    /// Initialize the UART hardware.
    ///
    /// This function needs to be run before the UART module is used.
    pub fn initialize(&self) {
        self.power_and_clock();
        self.enable_interrupts();
    }

    fn power_and_clock(&self) {
        prcm::Power::enable_domain(prcm::PowerDomain::Serial);
        while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}
        prcm::Clock::enable_uarts();
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        // Fractional baud rate divider
        let div = (((MCU_CLOCK * 8) / baud_rate) + 1) / 2;
        // Set the baud rate
        self.registers.ibrd.write(IntDivisor::DIVISOR.val(div / 64));
        self.registers
            .fbrd
            .write(FracDivisor::DIVISOR.val(div % 64));
    }

    fn fifo_enable(&self) {
        self.registers.lcrh.modify(LineControl::FIFO_ENABLE::SET);
    }

    fn fifo_disable(&self) {
        self.registers.lcrh.modify(LineControl::FIFO_ENABLE::CLEAR);
    }

    fn disable(&self) {
        // disable interrupts
        self.registers.imsc.write(Interrupts::ALL_INTERRUPTS::CLEAR);
        self.fifo_disable();
        self.registers.ctl.modify(
            Control::UART_ENABLE::CLEAR + Control::TX_ENABLE::CLEAR + Control::RX_ENABLE::CLEAR,
        );
    }

    fn enable_interrupts(&self) {
        // set only interrupts used
        self.registers
            .imsc
            .modify(Interrupts::TX::SET + Interrupts::END_OF_TRANSMISSION::SET);

        self.registers
            .ifls
            .modify(FifoLevelSelect::TXSEL::OneEighth);
        self.registers
            .ifls
            .modify(FifoLevelSelect::RXSEL::OneEighth);
    }

    /// Clears all interrupts related to UART.
    pub fn handle_events(&self) {
        // Clear interrupts
        self.registers.icr.write(Interrupts::ALL_INTERRUPTS::SET);

        self.rx.take().map(|rx| {
            if rx.index == rx.length || rx.newline {
                self.rx_client.map(move |client| {
                    self.registers
                        .imsc
                        .modify(Interrupts::RX::CLEAR + Interrupts::RX_TIMEOUT::CLEAR);
                    client.received_buffer(
                        rx.buffer,
                        rx.index,
                        ReturnCode::SUCCESS,
                        kernel::hil::uart::Error::None,
                    );
                });
            } else {
                self.rx.put(rx);
            }
        });

        self.tx.take().map(|tx| {
            if tx.index == tx.length {
                self.tx_client.map(move |client| {
                    client.transmitted_buffer(tx.buffer, tx.length, ReturnCode::SUCCESS);
                });
            } else {
                self.tx.put(tx);
            }
        });
    }

    pub fn write(&self, c: u32) {
        self.registers.dr.set(c);
    }

    // Pushes a byte into the TX FIFO.
    #[inline]
    pub fn send_byte(&self, c: u8) {
        // Put byte in data register
        self.registers.dr.set(c as u32);
    }

    // Pulls a byte out of the RX FIFO.
    #[inline]
    pub fn read_byte(&self) -> u8 {
        self.registers.dr.get() as u8
    }

    /// Checks if there is space in the transmit fifo queue.
    #[inline]
    pub fn rx_fifo_not_empty(&self) -> bool {
        !self.registers.fr.is_set(Flags::RX_FIFO_EMPTY)
    }

    /// Checks if there is space in the transmit fifo queue.
    #[inline]
    pub fn tx_fifo_not_full(&self) -> bool {
        !self.registers.fr.is_set(Flags::TX_FIFO_FULL)
    }
}

impl<'a> uart::Uart<'a> for UART<'a> {}
impl<'a> uart::UartData<'a> for UART<'a> {}

impl<'a> uart::Configure for UART<'a> {
    fn configure(&self, params: uart::Parameters) -> ReturnCode {
        // These could probably be implemented, but are currently ignored, so
        // throw an error.
        if params.stop_bits != uart::StopBits::One {
            return ReturnCode::ENOSUPPORT;
        }
        if params.parity != uart::Parity::None {
            return ReturnCode::ENOSUPPORT;
        }
        if params.hw_flow_control != false {
            return ReturnCode::ENOSUPPORT;
        }

        // Disable the UART before configuring
        self.disable();

        self.set_baud_rate(params.baud_rate);

        // Set word length
        let word_width = match params.width {
            uart::Width::Six => LineControl::WORD_LENGTH::Len6,
            uart::Width::Seven => LineControl::WORD_LENGTH::Len7,
            uart::Width::Eight => LineControl::WORD_LENGTH::Len8,
        };

        self.registers.lcrh.write(word_width);

        self.fifo_enable();

        self.enable_interrupts();

        // Enable UART, RX and TX
        self.registers
            .ctl
            .write(Control::UART_ENABLE::SET + Control::RX_ENABLE::SET + Control::TX_ENABLE::SET);

        ReturnCode::SUCCESS
    }
}

impl<'a> uart::Transmit<'a> for UART<'a> {
    fn set_transmit_client(&self, client: &'a uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        // if there is a weird input, don't try to do any transfers
        if len == 0 || len > buffer.len() {
            (ReturnCode::ESIZE, Some(buffer))
        } else if self.tx.is_some() {
            (ReturnCode::EBUSY, Some(buffer))
        } else {
            // we will send one byte, causing EOT interrupt
            if self.tx_fifo_not_full() {
                self.send_byte(buffer[0]);
            }
            // Transaction will be continued in interrupt bottom half
            self.tx.put(Transaction {
                buffer: buffer,
                length: len,
                index: 1,
                newline: false,
            });
            (ReturnCode::SUCCESS, None)
        }
    }

    // Incorporating this into the state machine is tricky because
    // it relies on implicit state from outstanding operations. I.e.,
    // rather than see if a TX interrupt occurred it checks if the FIFO
    // can accept data from a buffer. -pal 12/31/18
    fn transmit_word(&self, word: u32) -> ReturnCode {
        // if there's room in outgoing FIFO and no buffer transaction
        if self.tx_fifo_not_full() && self.tx.is_none() {
            self.write(word);
            return ReturnCode::SUCCESS;
        }
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}

impl<'a> uart::Receive<'a> for UART<'a> {
    fn set_receive_client(&self, client: &'a uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if len == 0 || len > buffer.len() {
            (ReturnCode::ESIZE, Some(buffer))
        } else if self.rx.is_some() || self.receiving_word.get() {
            (ReturnCode::EBUSY, Some(buffer))
        } else {
            self.registers
                .imsc
                .modify(Interrupts::RX::SET + Interrupts::RX_TIMEOUT::SET);

            self.rx.put(Transaction {
                buffer: buffer,
                length: len,
                index: 0,
                newline: false,
            });
            (ReturnCode::SUCCESS, None)
        }
    }

    fn receive_word(&self) -> ReturnCode {
        if self.rx.is_some() || self.receiving_word.get() {
            ReturnCode::EBUSY
        } else {
            self.receiving_word.set(true);
            ReturnCode::SUCCESS
        }
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}

impl<'a> PowerClient for UART<'a> {
    fn before_sleep(&self, _sleep_mode: u32) {
        prcm::Clock::disable_uarts();
    }

    fn after_wakeup(&self, _sleep_mode: u32) {
        unsafe {
            PM.request_resource(prcm::PowerDomain::Serial as u32);
        }

        self.initialize();
    }

    fn lowest_sleep_mode(&self) -> u32 {
        SleepMode::DeepSleep as u32
    }
}
