use core::cell::Cell;
use libtock_platform::{Callback, SubscribeData, SubscribeResponse};

pub struct FakeSyscalls<'k> {
    callback_pending: Cell<Option<usize>>,
    output: Cell<Vec<u8>>,
    write_buffer: Cell<Option<&'k [u8]>>,
    write_callback: Cell<Option<Box<dyn DynCallback + 'k>>>,
}

impl<'k> FakeSyscalls<'k> {
    pub fn new() -> Self {
        FakeSyscalls {
            callback_pending: Cell::new(None),
            output: Cell::new(Vec::new()),
            write_buffer: Cell::new(None),
            write_callback: Cell::new(None),
        }
    }

    pub fn read_buffer(&self) -> &'k [u8] {
        self.write_buffer.take().unwrap_or(&[])
    }
}

trait DynCallback {
    fn call(&self, arg1: usize, arg2: usize, arg3: usize);
}

struct RawCallback<C: Callback<SubscribeResponse<D>>, D: SubscribeData> {
    callback: C,
    data: D,
}

impl<C: Callback<SubscribeResponse<D>>, D: SubscribeData> DynCallback for RawCallback<C, D> {
    fn call(&self, arg1: usize, arg2: usize, arg3: usize) {
        self.callback.call(SubscribeResponse { arg1, arg2, arg3, data: self.data });
    }
}

impl<'k> libtock_platform::Syscalls<'k> for &FakeSyscalls<'k> {
    fn subscribe<C: Callback<SubscribeResponse<D>> + 'k, D: SubscribeData + 'k>(
        self, driver: usize, minor: usize, callback: C, data: D
    ) {
        if driver == 1 && minor == 1 {
            self.write_callback.set(Some(Box::new(RawCallback { callback, data })));
        }
    }

    unsafe fn raw_const_allow(self, major: usize, minor: usize, slice: *const u8, len: usize) {
        if major == 1 && minor == 1 {
            self.write_buffer.set(Some(core::slice::from_raw_parts(slice, len)));
        }
    }

    fn command(self, major: usize, minor: usize, arg1: usize, _arg2: usize) {
        if major != 1 || minor != 1 { return; }
        if let Some(buffer) = self.write_buffer.get() {
            let mut output = self.output.take();
            let bytes = core::cmp::min(arg1, buffer.len());
            output.extend_from_slice(&buffer[..bytes]);
            self.output.set(output);
            self.callback_pending.set(Some(bytes));
        }
    }

    fn yieldk(self) {
        let bytes = match self.callback_pending.take() {
            Some(bytes) => bytes,
            None => return,
        };
        if let Some(callback) = self.write_callback.take() {
            callback.call(bytes, 0, 0);
            self.write_callback.set(Some(callback));
        }
    }
}

//#[macro_export]
//macro_rules! test_component {
//	[$linkname: ident = $link:ident, $name:ident: $comp:ty = $init:expr] => {
//		let mut $name = $init;
//		#[derive(Clone, Copy)]
//		struct $link<'k> { component: &'k $comp }
//		impl<T> libtock_platform::Callback<T> for $link
//		where $comp: libtock_platform::Callback<T> {
//			fn locate() -> Self {
//				panic!("locate() unavailable for test components");
//			}
//
//			fn call(self, response: T) {
//				self.component.call(
//			}
//		}
//	};
//}
