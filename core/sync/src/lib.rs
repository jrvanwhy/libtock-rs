#![no_std]

use libtock_platform::{MethodCallback, Syscalls};

pub struct SyncAdapter<S, AsyncResponse> {
    response: core::cell::Cell<Option<AsyncResponse>>,
    syscalls: core::marker::PhantomData<S>,
}

impl<S, AsyncResponse> SyncAdapter<S, AsyncResponse> {
    pub const fn new() -> SyncAdapter<S, AsyncResponse> {
        SyncAdapter { response: core::cell::Cell::new(None),
                      syscalls: core::marker::PhantomData }
    }
}

impl<S: Syscalls, AsyncResponse> SyncAdapter<S, AsyncResponse> {
    pub fn wait(&self) -> AsyncResponse {
        loop {
            match self.response.take() {
                Some(response) => return response,
                None => S::yieldk(),
            }
        }
    }
}

impl<S, AsyncResponse> MethodCallback<AsyncResponse> for SyncAdapter<S, AsyncResponse> {
    fn call(&self, response: AsyncResponse) {
        self.response.set(Some(response));
    }
}
