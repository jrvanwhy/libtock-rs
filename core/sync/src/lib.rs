use libtock_platform::{Callback, Syscalls};

pub struct SyncAdapter<AsyncResponse> {
    response: core::cell::Cell<Option<AsyncResponse>>,
}

impl<AsyncResponse> SyncAdapter<AsyncResponse> {
    pub const fn new() -> SyncAdapter<AsyncResponse> {
        SyncAdapter { response: core::cell::Cell::new(None) }
    }

    pub fn wait<'k, S: Syscalls<'k>>(&self, syscalls: S) -> AsyncResponse {
        loop {
            match self.response.take() {
                Some(response) => return response,
                None => syscalls.yieldk(),
            }
        }
    }
}

impl<AsyncResponse> Callback<AsyncResponse> for &SyncAdapter<AsyncResponse> {
    fn call(self, response: AsyncResponse) {
        self.response.set(Some(response));
    }
}
