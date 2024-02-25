#![no_std]

use embedded_hal::spi::SpiDevice;
use smallvec::SmallVec;

pub struct ICMU<SPI> {
    spi: SPI,
    buf_tx: SmallVec<[u8; 256]>,
    buf_rx: SmallVec<[u8; 256]>,
}

#[repr(u8)]
enum Opcode {
    Activate = 0xB0,
    SdadTransmission = 0xA6,
    SdadStatus = 0xF5,
    ReadRegister = 0x97,
    WriteRegister = 0xD2,
    RegisterStatusData = 0xAD,
}

impl<SPI: SpiDevice> ICMU<SPI> {
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            buf_tx: SmallVec::new(),
            buf_rx: SmallVec::new(),
        }
    }

    pub fn activate(&mut self, active_vector: &[u8]) -> Result<(), SPI::Error> {
        self.buf_tx.push(Opcode::Activate as u8);
        self.buf_tx.extend_from_slice(active_vector);

        let ret = self.spi.write(&self.buf_tx);
        self.buf_tx.clear();
        ret
    }

    pub fn sdad_transmission(&mut self, data_rx: &mut [u8]) -> Result<(), SPI::Error> {
        let bufsize = data_rx.len() + 1;
        self.buf_tx.push(Opcode::SdadTransmission as u8);
        self.buf_tx.resize(bufsize, 0);
        self.buf_rx.resize(bufsize, 0);

        let ret = self.spi.transfer(&mut self.buf_rx, &self.buf_tx);
        data_rx.copy_from_slice(&self.buf_rx[1..bufsize]);
        self.buf_tx.clear();
        self.buf_rx.clear();
        ret
    }

    pub fn sdad_status(&mut self, svalid_vector: &mut [u8]) -> Result<(), SPI::Error> {
        let bufsize = svalid_vector.len() + 1;
        self.buf_tx.push(Opcode::SdadStatus as u8);
        self.buf_tx.resize(bufsize, 0);
        self.buf_rx.resize(bufsize, 0);

        let ret = self.spi.transfer(&mut self.buf_rx, &self.buf_tx);
        svalid_vector.copy_from_slice(&self.buf_rx[1..bufsize]);
        self.buf_tx.clear();
        self.buf_rx.clear();
        ret
    }

    pub fn read_register(&mut self, addr: u8) -> Result<(), SPI::Error> {
        self.buf_tx.push(Opcode::ReadRegister as u8);
        self.buf_tx.push(addr);

        let ret = self.spi.write(&self.buf_tx);
        self.buf_tx.clear();
        ret
    }

    pub fn write_register(&mut self, addr: u8, data_tx: u8) -> Result<(), SPI::Error> {
        self.buf_tx.push(Opcode::WriteRegister as u8);
        self.buf_tx.push(addr);
        self.buf_tx.push(data_tx);

        let ret = self.spi.write(&self.buf_tx);
        self.buf_tx.clear();
        ret
    }

    pub fn register_status_data(&mut self) -> Result<(u8, u8), SPI::Error> {
        self.buf_tx.push(Opcode::RegisterStatusData as u8);
        self.buf_tx.resize(3, 0);

        let ret = self.spi.transfer(&mut self.buf_rx, &self.buf_tx);
        let status_rx = self.buf_rx[1];
        let data_rx = self.buf_rx[2];
        self.buf_tx.clear();
        self.buf_rx.clear();
        ret?;
        Ok((status_rx, data_rx))
    }
}
