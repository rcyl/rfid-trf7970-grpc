pub const AGC: &str = "0109000304F0000000";
pub const AGC_RES: &str = "AGC Toggle";
pub const AGC_RES_2: &str = AGC;

pub const AM: &str = "0109000304F1FF0000";
pub const AM_RES: &str = "AM PM Toggle";
pub const AM_RES_2: &str = AM;

pub const EXT_ANT: &str = "01080003042B0000";
pub const EXT_ANT_RES: &str = EXT_ANT;

pub const ISO: &str = "010A0003041001210000";
pub const ISO_RES: &str = "Register write request.";

pub const _RF_HIGH_DATA: &str = "010C00030410002101020000";
pub const _RF_HIGH_DATA_RES: &str = "Register write request.";

pub const RF_HALF_DATA: &str = "010C00030410003101020000";
pub const RF_HALF_DATA_RES: &str = "Register write request.";

pub const INV_REQ: &str = "010B000304142601000000";
pub const UUID_REGEX: &str = r"\[[a-fA-F0-9]{16},[a-fA-F0-9]{2}\]";

pub const SINGLE_BLK_REGEX: &str = r"\[00[a-fA-F0-9]{8}\]";
pub const SINGLE_BLK_REQ: &str = "0113000304182220";
pub const SINGLE_BLK_REQ_END: &str = "0000";
pub const _SINGLE_BLK_REQ_ANS: &str = "Request mode.";
pub const SINGLE_BLK_START: &str = "[";
pub const SINGLE_BLK_OFFSET: usize = 3;
pub const SINGLE_BLK_CHARS: usize = 8;

pub const UUID_START: &str = "E0";
pub const UUID_CHARS: usize = 16;
pub const BLOCK_CHARS: usize = 2;
