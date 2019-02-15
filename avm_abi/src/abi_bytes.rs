use std::mem;

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

macro_rules! format_as_bytes {
    ($type_name: ident, $endian: expr, $len: expr) => {
        impl ToBytes for $type_name {
            fn to_bytes(&self) -> Vec<u8> {
                let bytes: [u8; $len] = match $endian {
                    0 => unsafe {mem::transmute(self.to_be())},
                    1 => unsafe {mem::transmute(self.to_le())},
                    _ => [0; $len]
                };
                bytes.to_vec()
            }
        }
    };
}

format_as_bytes!(u16, 0, 2);
format_as_bytes!(i16, 0, 2);
format_as_bytes!(u32, 0, 4);
format_as_bytes!(i32, 0, 4);
format_as_bytes!(u64, 0, 8);
format_as_bytes!(i64, 0, 8);

#[test]
fn test_as_bytes() {
    assert_eq!(0x10u16.to_bytes(), vec![0, 16]);
    assert_eq!(0x10i16.to_bytes(), vec![0, 16]);
    assert_eq!(65535u16.to_bytes(), vec![0xff, 0xff]);
    assert_eq!(32767i16.to_bytes(), vec![0x7f, 0xff]);
}