use libtock_platform::MethodCallback;

pub struct SyncAdapter<AsyncResponse, Syscalls> {
    response: core::cell::Cell<Option<AsyncResponse>>,
    syscalls: Syscalls,
}

impl<AsyncResponse, Syscalls> SyncAdapter<AsyncResponse, Syscalls> {
    pub const fn new(syscalls: Syscalls) -> SyncAdapter<AsyncResponse, Syscalls> {
        SyncAdapter { response: core::cell::Cell::new(None), syscalls }
    }
}

impl<AsyncResponse, Syscalls: libtock_platform::Syscalls> SyncAdapter<AsyncResponse, Syscalls> {
    pub fn wait(&self) -> AsyncResponse {
        loop {
            match self.response.take() {
                Some(response) => return response,
                None => self.syscalls.yieldk(),
            }
        }
    }
}

impl<AsyncResponse, Syscalls: libtock_platform::Syscalls>
MethodCallback<AsyncResponse> for SyncAdapter<AsyncResponse, Syscalls> {
    fn call(&self, response: AsyncResponse) {
        self.response.set(Some(response));
    }
}
