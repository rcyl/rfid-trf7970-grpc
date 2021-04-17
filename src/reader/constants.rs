const AGC: &'static str = "0109000304F0000000";
const AGC_RES: &'static str = "AGC Toggle";
const AGC_RES_2: &'static str = AGC;

const AM: &'static str = "0109000304F1FF0000";
const AM_RES: &'static str ="AM PM Toggle";
const AM_RES_2: &'static str = AM;

const EXT_ANT: &'static str = "01080003042B0000";
const EXT_ANT_RES: &'static str = EXT_ANT;

const ISO: &'static str = "010A0003041001210000";
const ISO_RES: &'static str = "Register write request.";

const _RF_HIGH_DATA: &'static str = "010C00030410002101020000";
const _RF_HIGH_DATA_RES: &'static str = "Register write request.";

const RF_HALF_DATA: &'static str = "010C00030410003101020000";
const RF_HALF_DATA_RES: &'static str = "Register write request.";

const INV_REQ: &'static str = "010B000304142601000000";
const UUID_REGEX: &'static str = r"\[[a-fA-F0-9]{16},[a-fA-F0-9]{2}\]";

const TRIES: i32 = 3;
