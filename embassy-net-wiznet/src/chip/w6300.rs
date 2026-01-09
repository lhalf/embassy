use embedded_hal_async::spi::{Operation, SpiDevice};

#[repr(u8)]
pub enum RegisterBlock {
    Common = 0x00,
    Socket0 = 0x01,
    TxBuf = 0x02,
    RxBuf = 0x03,
}

/// Wiznet W6300 chip.
pub enum W6300 {}

impl super::Chip for W6300 {}
impl super::SealedChip for W6300 {
    type Address = (RegisterBlock, u16);

    // W6300 Major Chip ID is 0x61 at offset 0x0000 (CIDR0)
    const CHIP_VERSION: u8 = 0x61;

    const COMMON_MODE: Self::Address = (RegisterBlock::Common, 0x2004); // SYCR0
    const COMMON_MAC: Self::Address = (RegisterBlock::Common, 0x4120); // SHAR
    const COMMON_SOCKET_INTR: Self::Address = (RegisterBlock::Common, 0x2114); // SIMR
    const COMMON_PHY_CFG: Self::Address = (RegisterBlock::Common, 0x3000); // PHYSR
    // W6300 Chip Identification Register 0 (CIDR0) is at 0x0000
    const COMMON_VERSION: Self::Address = (RegisterBlock::Common, 0x0000);

    const SOCKET_MODE: Self::Address = (RegisterBlock::Socket0, 0x0000); // Sn_MR
    const SOCKET_COMMAND: Self::Address = (RegisterBlock::Socket0, 0x0010); // Sn_CR
    const SOCKET_RXBUF_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0220); // Sn_RX_BSR
    const SOCKET_TXBUF_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0200); // Sn_TX_BSR
    const SOCKET_TX_FREE_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0204); // Sn_TX_FSR
    const SOCKET_TX_DATA_WRITE_PTR: Self::Address = (RegisterBlock::Socket0, 0x020C); // Sn_TX_WR
    const SOCKET_RECVD_SIZE: Self::Address = (RegisterBlock::Socket0, 0x0224); // Sn_RX_RSR
    const SOCKET_RX_DATA_READ_PTR: Self::Address = (RegisterBlock::Socket0, 0x0228); // Sn_RX_RD
    const SOCKET_INTR_MASK: Self::Address = (RegisterBlock::Socket0, 0x0024); // Sn_IMR
    const SOCKET_INTR: Self::Address = (RegisterBlock::Socket0, 0x0020); // Sn_IR
    const SOCKET_INTR_CLR: Self::Address = (RegisterBlock::Socket0, 0x0028); // Sn_IRCLR

    // MACRAW mode. See Page 55 of https://docs.wiznet.io/assets/files/20251204_W6300_DS_V101E-4f4cd2e75de8d76f51a741f6a492ea01.pdf.
    // Protocol mode (P[3:0]) for MACRAW is 0111 (0x07).
    // Note: Bit 7 is MAC Filter. Keeping consistent with W6100 implementation choice to disable it for now.
    const SOCKET_MODE_VALUE: u8 = 0b0000_0111;

    // W6300 has 32KB TX and 32KB RX memory.
    // Default allocation for 8 sockets is 4KB (0x1000) per socket.
    const BUF_SIZE: u16 = 0x1000;
    const AUTO_WRAP: bool = true;

    fn rx_addr(addr: u16) -> Self::Address {
        (RegisterBlock::RxBuf, addr)
    }

    fn tx_addr(addr: u16) -> Self::Address {
        (RegisterBlock::TxBuf, addr)
    }

    async fn bus_read<SPI: SpiDevice>(
        spi: &mut SPI,
        address: Self::Address,
        data: &mut [u8],
    ) -> Result<(), SPI::Error> {
        // W6300 SPI Frame Layout (Datasheet section 5.1.1):
        // Instruction (8 bits) | Address (16 bits) | Dummy (8 bits) | Data (N bits)

        // Instruction Phase:
        // Bit 7-6: MOD[1:0] (00 for Single SPI)
        // Bit 5:   RWB (0 for Read)
        // Bit 4-0: BSB (Block Select Bits) -> matches RegisterBlock value
        let instruction_phase = [address.0 as u8];
        let address_phase = address.1.to_be_bytes();
        let dummy_phase = [0u8]; // Fixed 8-bit dummy phase

        let operations = &mut [
            Operation::Write(&instruction_phase),
            Operation::Write(&address_phase),
            Operation::Write(&dummy_phase),
            Operation::TransferInPlace(data),
        ];
        spi.transaction(operations).await
    }

    async fn bus_write<SPI: SpiDevice>(spi: &mut SPI, address: Self::Address, data: &[u8]) -> Result<(), SPI::Error> {
        // W6300 SPI Frame Layout (Datasheet section 5.1.1):
        // Instruction (8 bits) | Address (16 bits) | Dummy (8 bits) | Data (N bits)

        // Instruction Phase:
        // Bit 7-6: MOD[1:0] (00 for Single SPI)
        // Bit 5:   RWB (1 for Write)
        // Bit 4-0: BSB (Block Select Bits) -> matches RegisterBlock value
        // 0x20 sets the RWB bit to 1.
        let instruction_phase = [(address.0 as u8) | 0x20];
        let address_phase = address.1.to_be_bytes();
        let dummy_phase = [0u8]; // Fixed 8-bit dummy phase

        let operations = &mut [
            Operation::Write(&instruction_phase),
            Operation::Write(&address_phase),
            Operation::Write(&dummy_phase),
            Operation::Write(data),
        ];
        spi.transaction(operations).await
    }
}
