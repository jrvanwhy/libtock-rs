use libtock_fmt::TDebug;
use libtock_tdebug_derive::TDebug;

// A Writer that logs all its write calls.
#[derive(Default)]
struct TestWriter {
    calls: std::cell::Cell<Vec<String>>,
}

impl libtock_fmt::Writer for &TestWriter {
    type Error = std::convert::Infallible;

    fn write(self, buffer: &[u8]) -> Result<(), Self::Error> {
        let mut calls = self.calls.take();
        calls.push(String::from_utf8(buffer.iter().map(|&v| v).collect()).expect("Bad UTF-8"));
        self.calls.set(calls);
        Ok(())
    }
}

//#[derive(TDebug)]
//struct Tuple(u8);

#[derive(TDebug)]
union TestUnion {
    _a: u8,
}

#[test]
fn union_print() {
    let writer = Default::default();
    let union = TestUnion { _a: 3 };
    assert!(union.fmt(&writer).is_ok());
    assert_eq!(writer.calls.take(), ["TestUnion{...}"]);
}
