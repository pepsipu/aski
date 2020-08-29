use crate::stream::IStream;

pub fn is_ws(c: u8, _is: Option<&mut IStream>) -> bool {
    match c {
        b' ' | b'\t' => true,
        _ => false,
    }
}

pub fn is_int(c: u8, _is: Option<&mut IStream>) -> bool {
    match c {
        b'0' ..= b'9' => true,
        _ => false,
    }
}

pub fn is_quote(c: u8, _is: Option<&mut IStream>) -> bool {
    c == b'"'
}

pub fn is_kw_or_var(c: u8, _is: Option<&mut IStream>) -> bool {
    match c {
        b'a' ..= b'z' | b'A' ..= b'Z' | b'_' => true,
        _ => false,
    }
}

pub fn is_op(c: u8, _is: Option<&mut IStream>) -> bool {
    match c {
        b'=' | b'-' | b'+' | b'/' | b'*' | b'%' | b'!' => true,
        _ => false,
    }
}

pub fn is_register_name(c: u8, _is: Option<&mut IStream>) -> bool {
    match c {
        b'a' ..= b'z' => true,
        _ => false,
    }
}

