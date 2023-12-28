use crate::{allow_rw_num, command_num, subscribe_num, DRIVER_NUM};
use core::cell::Cell;
use core::marker::PhantomData;
use libtock_platform::allow_rw::AllowRw;
use libtock_platform::share::Handle;
use libtock_platform::{
    allow_rw, subscribe, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

// -----------------------------------------------------------------------------
// AsyncReader
// -----------------------------------------------------------------------------

pub struct AsyncReader<S: Syscalls, C: allow_rw::Config + subscribe::Config = DefaultConfig> {
    _config: PhantomData<C>,
    output: Cell<Option<Output>>,
    _syscalls: PhantomData<S>,
}

impl<S: Syscalls, C: allow_rw::Config + subscribe::Config> Default for AsyncReader<S, C> {
    fn default() -> Self {
        AsyncReader {
            _config: PhantomData,
            output: Cell::new(None),
            _syscalls: PhantomData,
        }
    }
}

impl<S: Syscalls, C: allow_rw::Config + subscribe::Config> AsyncReader<S, C> {
    pub fn read<'share>(
        &'share self,
        handle: Handle<Handles<'share, S>>,
        buffer: &'share mut [u8],
    ) -> Result<(), ErrorCode> {
        let (allow_rw, subscribe) = handle.split();
        let len = buffer.len();
        S::allow_rw::<C, DRIVER_NUM, { allow_rw_num::READ }>(allow_rw, buffer)?;
        S::subscribe::<_, _, C, DRIVER_NUM, { subscribe_num::READ }>(subscribe, self)?;
        S::command(DRIVER_NUM, command_num::READ, len as u32, 0).to_result()
    }

    pub fn done(&self) -> bool {
        self.output.get().is_some()
    }

    pub fn output(&self) -> Option<Output> {
        self.output.get()
    }

    pub fn reset(&self) {
        self.output.set(None);
    }
}

impl<S: Syscalls, C: allow_rw::Config + subscribe::Config>
    Upcall<subscribe::OneId<DRIVER_NUM, { subscribe_num::READ }>> for AsyncReader<S, C>
{
    fn upcall(&self, status: u32, bytes: u32, _: u32) {
        let status = match status {
            0 => Ok(()),
            error => Err(error.try_into().unwrap_or(ErrorCode::Fail)),
        };
        self.output.set(Some(Output { status, bytes }));
    }
}

// -----------------------------------------------------------------------------
// Other types
// -----------------------------------------------------------------------------

pub type Handles<'share, S> = (
    AllowRw<'share, S, DRIVER_NUM, { allow_rw_num::READ }>,
    Subscribe<'share, S, DRIVER_NUM, { subscribe_num::READ }>,
);

#[derive(Clone, Copy)]
pub struct Output {
    pub bytes: u32,
    pub status: Result<(), ErrorCode>,
}
