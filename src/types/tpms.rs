use super::{tpm, tpma, Handle};
use crate::{
    error::{MarshalError, UnmarshalError},
    marshal::{pop_array_mut, pop_slice},
    polyfill::ToArr,
    Fixed, Marshal, Unmarshal,
};

// TPMS_TIME_INFO
#[derive(Clone, Copy, Debug, Default)]
pub struct TimeInfo {
    time: u64,
    clock_info: ClockInfo,
}
impl Fixed for TimeInfo {
    const SIZE: usize = <u64 as Fixed>::SIZE + <ClockInfo as Fixed>::SIZE;
    type ARRAY = [u8; Self::SIZE];
    fn marshal_fixed(&self, arr: &mut Self::ARRAY) {
        self.time.marshal_fixed(arr[0..8].to_arr());
        self.clock_info.marshal_fixed(arr[8..].to_arr());
    }
}
impl Unmarshal<'_> for TimeInfo {
    fn unmarshal(&mut self, buf: &mut &[u8]) -> Result<(), UnmarshalError> {
        self.time.unmarshal(buf)?;
        self.clock_info.unmarshal(buf)
    }
}

// TPMS_CLOCK_INFO
#[derive(Clone, Copy, Debug, Default)]
pub struct ClockInfo {
    clock: u64,
    reset_count: u32,
    restart_count: u32,
    safe: bool,
}

impl Fixed for ClockInfo {
    const SIZE: usize =
        <u64 as Fixed>::SIZE + <u32 as Fixed>::SIZE + <u32 as Fixed>::SIZE + <bool as Fixed>::SIZE;
    type ARRAY = [u8; Self::SIZE];

    fn marshal_fixed(&self, arr: &mut Self::ARRAY) {
        self.clock.marshal_fixed(arr[0..8].to_arr());
        self.reset_count.marshal_fixed(arr[8..12].to_arr());
        self.restart_count.marshal_fixed(arr[12..16].to_arr());
        self.safe.marshal_fixed(arr[16..].to_arr());
    }
}
impl Unmarshal<'_> for ClockInfo {
    fn unmarshal(&mut self, buf: &mut &[u8]) -> Result<(), UnmarshalError> {
        self.clock.unmarshal(buf)?;
        self.reset_count.unmarshal(buf)?;
        self.restart_count.unmarshal(buf)?;
        self.safe.unmarshal(buf)
    }
}

// TPMS_AUTH_COMMAND
#[derive(Clone, Copy, Default)]
pub struct AuthCommand<'a> {
    pub session_handle: Handle,
    pub nonce: &'a [u8],
    pub session_attributes: tpma::Session,
    pub hmac: &'a [u8],
}

impl Marshal for AuthCommand<'_> {
    fn marshal(&self, buf: &mut &mut [u8]) -> Result<(), MarshalError> {
        self.session_handle.marshal(buf)?;
        self.nonce.marshal(buf)?;
        self.session_attributes.marshal(buf)?;
        self.hmac.marshal(buf)?;
        Ok(())
    }
}

// TPMS_AUTH_RESPONSE
#[derive(Clone, Copy, Default)]
pub struct AuthResponse<'a> {
    pub nonce: &'a [u8],
    pub session_attributes: tpma::Session,
    pub hmac: &'a [u8],
}

impl<'a> Unmarshal<'a> for AuthResponse<'a> {
    fn unmarshal(&mut self, buf: &mut &'a [u8]) -> Result<(), UnmarshalError> {
        self.nonce.unmarshal(buf)?;
        self.session_attributes.unmarshal(buf)?;
        self.hmac.unmarshal(buf)?;
        Ok(())
    }
}

const SIZE_OF_SELECT: usize = 3;
pub const NUM_PCRS: usize = 8 * SIZE_OF_SELECT;

// TODO: Do we want to support anything other than 24 PCRs?
pub type PcrSelect = [bool; NUM_PCRS];

impl Marshal for PcrSelect {
    fn marshal(&self, buf: &mut &mut [u8]) -> Result<(), MarshalError> {
        (SIZE_OF_SELECT as u8).marshal(buf)?;
        let sel = pop_array_mut(buf)?;

        *sel = [0; SIZE_OF_SELECT];
        for (i, &bit) in self.iter().enumerate() {
            if !bit {
                continue;
            }
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            sel[byte_idx] |= 1 << bit_idx;
        }
        Ok(())
    }
}

impl Unmarshal<'_> for PcrSelect {
    fn unmarshal(&mut self, buf: &mut &[u8]) -> Result<(), UnmarshalError> {
        let size: usize = u8::unmarshal_val(buf)?.into();
        let sel = pop_slice(size, buf)?;

        *self = [false; NUM_PCRS];
        for (byte_idx, &byte) in sel.iter().enumerate() {
            for bit_idx in 0..8 {
                let pcr_num = 8 * byte_idx + bit_idx;
                if byte & (1 << bit_idx) == 0 {
                    continue;
                }
                if pcr_num > NUM_PCRS {
                    return Err(UnmarshalError::PcrTooLarge(pcr_num));
                }
                self[pcr_num] = true;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct PcrSelection {
    pub hash: tpm::Alg,
    pub select: PcrSelect,
}

impl Marshal for PcrSelection {
    fn marshal(&self, buf: &mut &mut [u8]) -> Result<(), MarshalError> {
        self.hash.marshal(buf)?;
        self.select.marshal(buf)
    }
}

impl Unmarshal<'_> for PcrSelection {
    fn unmarshal(&mut self, buf: &mut &[u8]) -> Result<(), UnmarshalError> {
        self.hash.unmarshal(buf)?;
        self.select.unmarshal(buf)
    }
}
