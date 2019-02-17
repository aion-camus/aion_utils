
#![allow(unused)]

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

enum AbiToken<'a> {
    UCHAR(u8),
    BOOL(bool),
    INT8(i8),
    INT16(i16),
    INT32(i32),
    INT64(i64),
    FLOAT(f32),
    DOUBLE(f64),
    AUCHAR(&'a [u8]),
    ABOOL(&'a [bool]),
    AINT8(&'a [i8]),
    AINT16(&'a [i16]),
    AINT32(&'a [i32]),
    AINT64(&'a [i64]),
    AFLOAT(&'a [f32]),
    ADOUBLE(&'a [f64]),
    STRING(String),
}

trait AVMEncoder {
    fn encode(&self) -> Vec<u8>;
}

impl<'a> AVMEncoder for AbiToken<'a> {
    fn encode(&self) -> Vec<u8> {
        let mut res = Vec::new();
        match *self {
            AbiToken::UCHAR(v) => {
                res.push(0x01);
                res.push(v);
            },
            AbiToken::BOOL(v) => {
                res.push(0x02);
                if v {
                    res.push(0x01);
                } else {
                    res.push(0x0);
                }
            },
            AbiToken::INT8(v) => {
                res.push(0x03);
                res.push(v as u8);
            },
            AbiToken::INT16(v) => {
                res.push(0x04);
                res.append(&mut v.to_bytes())
            },
            AbiToken::INT32(v) => {
                res.push(0x05);
                res.append(&mut v.to_bytes())
            },
            AbiToken::INT64(v) => {
                res.push(0x06);
                res.append(&mut v.to_bytes())
            },
            AbiToken::FLOAT(v) => {
                res.push(0x07);
                //TODO: float variable
                panic!("unsupported for float");
            },
            AbiToken::DOUBLE(v) => {
                res.push(0x08);
                //TODO: double variable
                panic!("unsupported for double");
            },
            AbiToken::AUCHAR(v) => {
                res.push(0x11);
                v.iter().map(|v| {
                    res.push(*v)
                });
            },
            AbiToken::ABOOL(v) => {
                res.push(0x12);
                v.iter().map(|v| {
                    if *v {
                        res.push(0x01)
                    } else {
                        res.push(0x02)
                    }
                });
            },
            AbiToken::AINT8(v) => {
                res.push(0x13);
                v.iter().map(|v| {
                    res.push(*v as u8)
                });
            },
            AbiToken::AINT16(v) => {
                res.push(0x14);
                v.iter().map(|v| {
                    res.append(&mut v.to_bytes())
                });
            },
            AbiToken::AINT32(v) => {
                res.push(0x15);
                v.iter().map(|v| {
                    res.append(&mut v.to_bytes())
                });
            },
            AbiToken::AINT64(v) => {
                res.push(0x16);
                v.iter().map(|v| {
                    res.append(&mut v.to_bytes())
                });
            },
            AbiToken::AFLOAT(v) => {
                res.push(0x17);
                //TODO: float array
                panic!("unsupported for float array");
            },
            AbiToken::ADOUBLE(v) => {
                res.push(0x18);
                panic!("unsupported for double array");
            },
            AbiToken::STRING(ref v) => {
                res.push(0x21);
                res.append(&mut (v.len() as i32).to_bytes());
                res.append(&mut v.clone().into_bytes());
            },
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        let token = AbiToken::STRING("hello".to_string());

        assert_eq!(token.encode(), vec![33,0,0,0,5,104,101,108,108,111]);
    }
}