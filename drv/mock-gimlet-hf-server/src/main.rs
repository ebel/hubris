// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Gimlet host flash server.
//!
//! This server is responsible for managing access to the host flash; it embeds
//! the QSPI flash driver.

#![no_std]
#![no_main]

use drv_gimlet_hf_api::{HfDevSelect, HfError, HfMuxState, PAGE_SIZE_BYTES};
use drv_hash_api::SHA256_SZ;
use idol_runtime::{ClientError, Leased, LenLimit, RequestError, R, W};
use userlib::RecvMessage;

#[export_name = "main"]
fn main() -> ! {
    let mut buffer = [0; idl::INCOMING_SIZE];
    let mut server = ServerImpl {
        capacity: 16 << 20, // Claim to have 16 MiB
        mux_state: HfMuxState::SP,
        dev_state: HfDevSelect::Flash0,
    };

    loop {
        idol_runtime::dispatch(&mut buffer, &mut server);
    }
}

struct ServerImpl {
    capacity: usize,

    /// Selects between the SP and SP3 talking to the QSPI flash
    mux_state: HfMuxState,

    /// Selects between QSPI flash chips 1 and 2 (if present)
    dev_state: HfDevSelect,
}

impl idl::InOrderHostFlashImpl for ServerImpl {
    fn read_id(
        &mut self,
        _: &RecvMessage,
    ) -> Result<[u8; 20], RequestError<HfError>> {
        Ok(*b"mockmockmockmockmock")
    }

    fn capacity(
        &mut self,
        _: &RecvMessage,
    ) -> Result<usize, RequestError<HfError>> {
        Ok(self.capacity)
    }

    fn read_status(
        &mut self,
        _: &RecvMessage,
    ) -> Result<u8, RequestError<HfError>> {
        Ok(0)
    }

    fn bulk_erase(
        &mut self,
        _: &RecvMessage,
    ) -> Result<(), RequestError<HfError>> {
        Ok(())
    }

    fn page_program(
        &mut self,
        _: &RecvMessage,
        _addr: u32,
        _data: LenLimit<Leased<R, [u8]>, PAGE_SIZE_BYTES>,
    ) -> Result<(), RequestError<HfError>> {
        Ok(())
    }

    fn read(
        &mut self,
        _: &RecvMessage,
        _addr: u32,
        dest: LenLimit<Leased<W, [u8]>, PAGE_SIZE_BYTES>,
    ) -> Result<(), RequestError<HfError>> {
        let zero = [0; PAGE_SIZE_BYTES];

        dest.write_range(0..dest.len(), &zero[..dest.len()])
            .map_err(|_| RequestError::Fail(ClientError::WentAway))?;

        Ok(())
    }

    fn sector_erase(
        &mut self,
        _: &RecvMessage,
        _addr: u32,
    ) -> Result<(), RequestError<HfError>> {
        Ok(())
    }

    fn get_mux(
        &mut self,
        _: &RecvMessage,
    ) -> Result<HfMuxState, RequestError<HfError>> {
        Ok(self.mux_state)
    }

    fn set_mux(
        &mut self,
        _: &RecvMessage,
        state: HfMuxState,
    ) -> Result<(), RequestError<HfError>> {
        self.mux_state = state;
        Ok(())
    }

    fn get_dev(
        &mut self,
        _: &RecvMessage,
    ) -> Result<HfDevSelect, RequestError<HfError>> {
        Ok(self.dev_state)
    }

    fn set_dev(
        &mut self,
        _: &RecvMessage,
        state: HfDevSelect,
    ) -> Result<(), RequestError<HfError>> {
        self.dev_state = state;
        Ok(())
    }

    #[cfg(feature = "hash")]
    fn hash(
        &mut self,
        _: &RecvMessage,
        _addr: u32,
        _len: u32,
    ) -> Result<[u8; SHA256_SZ], RequestError<HfError>> {
        Ok(*b"mockmockmockmockmockmockmockmock")
    }

    #[cfg(not(feature = "hash"))]
    fn hash(
        &mut self,
        _: &RecvMessage,
        _addr: u32,
        _len: u32,
    ) -> Result<[u8; SHA256_SZ], RequestError<HfError>> {
        Err(HfError::HashNotConfigured.into())
    }
}

mod idl {
    use super::{HfDevSelect, HfError, HfMuxState};

    include!(concat!(env!("OUT_DIR"), "/server_stub.rs"));
}
