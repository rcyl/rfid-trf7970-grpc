pub const AGC: &'static str = "0109000304F0000000";
pub const AGC_RES: &'static str = "AGC Toggle";
pub const AGC_RES_2: &'static str = AGC;

pub const AM: &'static str = "0109000304F1FF0000";
pub const AM_RES: &'static str ="AM PM Toggle";
pub const AM_RES_2: &'static str = AM;

pub const EXT_ANT: &'static str = "01080003042B0000";
pub const EXT_ANT_RES: &'static str = EXT_ANT;

pub const ISO: &'static str = "010A0003041001210000";
pub const ISO_RES: &'static str = "Register write request.";

pub const _RF_HIGH_DATA: &'static str = "010C00030410002101020000";
pub const _RF_HIGH_DATA_RES: &'static str = "Register write request.";

pub const RF_HALF_DATA: &'static str = "010C00030410003101020000";
pub const RF_HALF_DATA_RES: &'static str = "Register write request.";

pub const INV_REQ: &'static str = "010B000304142601000000";
pub const UUID_REGEX: &'static str = r"\[[a-fA-F0-9]{16},[a-fA-F0-9]{2}\]";

pub const SINGLE_BLK_REGEX: &'static str = r"\\[00[a-fA-F0-9]{8}\\]";
pub const UUID_START: &'static str = "E0";
pub const UUID_CHARS: usize = 16;

pub const TRIES: u32 = 3;
