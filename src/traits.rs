//! Main traits for interacting with and using a TPM2 Device.
use crate::{
    commands::{run_command, GetRandom},
    error::DriverError,
    types::Auth,
    Command, Error,
};

/// A TPM2 Device
pub trait Tpm {
    fn command_buf(&mut self) -> &mut [u8];
    fn response_buf(&self) -> &[u8];
    fn execute_command(&mut self, cmd_size: u32) -> Result<(), DriverError>;
}

/// Trait extending [`Tpm`] for running raw commands.
pub trait TpmRaw: Tpm {
    #[inline]
    fn run<'a, C: Command>(&'a mut self, cmd: &C) -> Result<C::Response<'a>, Error> {
        self.run_with_auths(cmd, &[])
    }

    #[inline]
    fn run_with_auths<'a, C: Command>(
        &'a mut self,
        cmd: &C,
        auths: &[&dyn Auth],
    ) -> Result<C::Response<'a>, Error> {
        let mut rsp: C::Response<'a> = Default::default();
        run_command(self.as_tpm(), C::CODE, auths, cmd, &mut rsp)?;
        Ok(rsp)
    }

    #[doc(hidden)]
    fn as_tpm(&mut self) -> &mut dyn Tpm;
}

impl<T: Tpm> TpmRaw for T {
    #[inline]
    fn as_tpm(&mut self) -> &mut dyn Tpm {
        self
    }
}
impl TpmRaw for dyn Tpm + '_ {
    #[inline]
    fn as_tpm(&mut self) -> &mut dyn Tpm {
        self
    }
}

/// Trait extending [`Tpm`] for running higher-level TPM workflows.
///
/// These methods almost always issues multiple TPM commands under the hood.
pub trait TpmExt: Tpm {
    fn getrandom(&mut self, buf: &mut [u8]) -> Result<(), Error>;
}

impl<T: TpmRaw + ?Sized> TpmExt for T {
    fn getrandom(&mut self, mut buf: &mut [u8]) -> Result<(), Error> {
        while !buf.is_empty() {
            let bytes_requested = buf.len().try_into().unwrap_or(u16::MAX);
            let rsp = self.run(&GetRandom { bytes_requested })?;

            let bytes_received = rsp.random_bytes.len();
            buf[..bytes_received].copy_from_slice(rsp.random_bytes);
            buf = &mut buf[bytes_received..];
        }
        Ok(())
    }
}