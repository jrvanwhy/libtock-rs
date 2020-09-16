use core::cell::Cell;

type RawCallback = extern "C" fn(usize, usize, usize, usize);

pub struct FakeSyscalls {
    callback_pending: Cell<Option<usize>>,
    output: Cell<Vec<u8>>,
    write_buffer: Cell<Option<&'static [u8]>>,
    write_callback: Cell<Option<RawCallback>>,
    write_data: Cell<usize>,
}

impl FakeSyscalls {
    pub fn new() -> Self {
        FakeSyscalls {
            callback_pending: Cell::new(None),
            output: Cell::new(Vec::new()),
            write_buffer: Cell::new(None),
            write_callback: Cell::new(None),
            write_data: Cell::new(0),
        }
    }

    pub fn read_buffer(&self) -> &'static [u8] {
        self.write_buffer.take().unwrap_or(&[])
    }
}

impl libtock_platform::Syscalls for &FakeSyscalls {
    fn subscribe(self, driver: usize, minor: usize, callback: extern "C" fn(usize, usize, usize, usize), data: usize) {
        if driver == 1 && minor == 1 {
            self.write_callback.set(Some(callback));
            self.write_data.set(data);
        }
    }

    unsafe fn const_allow(self, major: usize, minor: usize, slice: *const u8, len: usize) {
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
        if let Some(callback) = self.write_callback.get() {
            callback(bytes, 0, 0, self.write_data.get());
        }
    }
}

#[macro_export]
macro_rules! test_component {
    [$link:ident, $name:ident: $comp:ty = $init:expr] => {
        let $name = std::boxed::Box::leak(std::boxed::Box::new($init)) as &$comp;
        std::thread_local!(static GLOBAL: std::cell::Cell<Option<&'static $comp>>
                           = std::cell::Cell::new(None));
        GLOBAL.with(|g| g.set(Some($name)));
        struct $link;
        impl<T> libtock_platform::FreeCallback<T> for $link
        where &'static $comp: libtock_platform::MethodCallback<T> {
            fn call(response: T) {
                GLOBAL.with(|g| g.get().unwrap()).call(response);
            }
        }
    };
}
