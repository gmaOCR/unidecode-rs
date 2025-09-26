pub fn lookup(cp: u32) -> Option<&'static str> {
    match cp {
        128624u32 => Some("et"),
        128625u32 => Some("et"),
        128626u32 => Some("et"),
        128627u32 => Some("et"),
        128628u32 => Some("&"),
        128629u32 => Some("&"),
        128630u32 => Some("\""),
        128631u32 => Some("\""),
        128632u32 => Some(",,"),
        128633u32 => Some("!?"),
        128634u32 => Some("!?"),
        128635u32 => Some("!?"),
        _ => None,
    }
}
