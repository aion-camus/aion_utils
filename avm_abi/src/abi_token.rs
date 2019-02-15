
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
                res.append(v.to_bytes())
            },
            AbiToken::INT32(v) => {
                res.push(0x05);
            },
            AbiToken::INT64(v) => {
                res.push(0x06);
            },
            AbiToken::FLOAT(v) => {
                res.push(0x07);
            },
            AbiToken::DOUBLE(v) => {
                res.push(0x08);
            },
            AbiToken::AUCHAR(v) => {
                res.push(0x11);
            },
            AbiToken::ABOOL(v) => {
                res.push(0x12);
            },
            AbiToken::AINT8(v) => {
                res.push(0x13);
            },
            AbiToken::AINT16(v) => {
                res.push(0x14);
            },
            AbiToken::AINT32(v) => {
                res.push(0x15);
            },
            AbiToken::AINT64(v) => {
                res.push(0x16);
            },
            AbiToken::AFLOAT(v) => {
                res.push(0x17);
            },
            AbiToken::ADOUBLE(v) => {
                res.push(0x18)
            },
            AbiToken::STRING(v) => {
                res.push(0x21);
            },
        }

        res
    }
}


// impl TokenType {
//     fn encode(&self) -> Option<u8> {
//         match *self {
//             TokenType::UCHAR => Some(0x01),
//             TokenType::BOOL => Some(0x02),
//             TokenType::INT8 => Some(0x03),
//             TokenType::INT16 => Some(0x04),
//             TokenType::INT32 => Some(0x05),
//             TokenType::INT64 => Some(0x06),
//             TokenType::FLOAT => Some(0x07),
//             TokenType::DOUBLE => Some(0x08),
//             TokenType::AUCHAR => Some(0x11),
//             TokenType::ABOOL => Some(0x12),
//             TokenType::AINT8 => Some(0x13),
//             TokenType::AINT16 => Some(0x14),
//             TokenType::AINT32 => Some(0x15),
//             TokenType::AINT64 => Some(0x16),
//             TokenType::AFLOAT => Some(0x17),
//             TokenType::ADOUBLE => Some(0x18),
//             TokenType::STRING => Some(0x21),
//         }
//     }
// }