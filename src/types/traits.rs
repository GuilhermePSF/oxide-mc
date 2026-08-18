use crate::error::MineError;
use std::io::{Read, Write};

pub trait McRead: Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError>;
}

pub trait McWrite {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError>;
}
