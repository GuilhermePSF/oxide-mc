use super::traits::{McRead, McWrite};
use crate::error::MineError;
use std::io::{Read, Write};

impl McRead for bool {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let byte = u8::read(reader)?;
        Ok(byte != 0)
    }
}

impl McWrite for bool {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        let byte = if *self { 1u8 } else { 0u8 };
        byte.write(writer)
    }
}

impl McRead for u8 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl McWrite for u8 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&[*self])?;
        Ok(())
    }
}

impl McRead for i8 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let byte = u8::read(reader)?;
        Ok(byte as i8)
    }
}

impl McWrite for i8 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        (*self as u8).write(writer)
    }
}

impl McRead for u16 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
}

impl McWrite for u16 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl McRead for i16 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }
}

impl McWrite for i16 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl McRead for u32 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl McWrite for u32 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl McRead for i32 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }
}

impl McWrite for i32 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl McRead for u64 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }
}

impl McWrite for u64 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl McRead for i64 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }
}

impl McWrite for i64 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl McRead for f32 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }
}

impl McWrite for f32 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

impl McRead for f64 {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }
}

impl McWrite for f64 {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        writer.write_all(&self.to_be_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_primitives_roundtrip() {
        let mut buf = Vec::new();

        true.write(&mut buf).unwrap();
        false.write(&mut buf).unwrap();
        42u8.write(&mut buf).unwrap();
        (-17i8).write(&mut buf).unwrap();
        25565u16.write(&mut buf).unwrap();
        (-12345i16).write(&mut buf).unwrap();
        100_000u32.write(&mut buf).unwrap();
        (-999_999i32).write(&mut buf).unwrap();
        1234567890123u64.write(&mut buf).unwrap();
        (-9876543210987i64).write(&mut buf).unwrap();
        123.456f32.write(&mut buf).unwrap();
        (-987.654321f64).write(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        assert_eq!(bool::read(&mut cursor).unwrap(), true);
        assert_eq!(bool::read(&mut cursor).unwrap(), false);
        assert_eq!(u8::read(&mut cursor).unwrap(), 42);
        assert_eq!(i8::read(&mut cursor).unwrap(), -17);
        assert_eq!(u16::read(&mut cursor).unwrap(), 25565);
        assert_eq!(i16::read(&mut cursor).unwrap(), -12345);
        assert_eq!(u32::read(&mut cursor).unwrap(), 100_000);
        assert_eq!(i32::read(&mut cursor).unwrap(), -999_999);
        assert_eq!(u64::read(&mut cursor).unwrap(), 1234567890123);
        assert_eq!(i64::read(&mut cursor).unwrap(), -9876543210987);
        assert_eq!(f32::read(&mut cursor).unwrap(), 123.456f32);
        assert_eq!(f64::read(&mut cursor).unwrap(), -987.654321f64);
    }
}
