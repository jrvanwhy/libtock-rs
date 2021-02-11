/// `TDebug` is `libtock`'s version of `core::fmt::Debug`. The formatting
/// machinery in `core::fmt` is very heavyweight -- in particular, it uses
/// dynamic dispatch in a way that defeats LLVM's devirtualization and causes
/// several kilobytes of code to be unnecessarily included.

pub trait TDebug {
    fn fmt<W: crate::Writer>(&self, writer: W) -> Result<(), W::Error>;
}
