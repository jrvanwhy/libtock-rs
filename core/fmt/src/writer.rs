/// A `Writer` receives buffers and writes them somewhere. A writer is passed
/// into `TDebug::fmt`.

pub trait Writer: Copy {
    type Error;

    fn write(self, buffer: &[u8]) -> Result<(), Self::Error>;
}
