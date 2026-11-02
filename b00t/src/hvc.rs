pub const HVC_VERSION: u32 = 0x0001_0000;
pub const HVC_GET_VERSION: u64 = 0x0000;
pub const HVC_CONSOLE_PUTS: u64 = 0x0001;
pub const HVC_RTC_GET: u64 = 0x0010;
pub const HVC_RTC_SET: u64 = 0x0011;
pub const HVC_RNG_READ: u64 = 0x0020;
pub const HVC_RESET: u64 = 0x0030;
pub const HVC_SHUTDOWN: u64 = 0x0031;
pub const HVC_BLOCK_READ: u64 = 0x0100;
pub const HVC_BLOCK_WRITE: u64 = 0x0101;
pub const HVC_BLOCK_FLUSH: u64 = 0x0102;
pub const HVC_BLOCK_CAPACITY: u64 = 0x0103;

pub const HVC_SUCCESS: i64 = 0;
pub const HVC_NOT_SUPPORTED: i64 = -1;
pub const HVC_INVALID_PARAMETER: i64 = -2;
pub const HVC_IO_ERROR: i64 = -3;
pub const HVC_BUSY: i64 = -4;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BlockRequest {
    pub lba: u64,
    pub block_count: u32,
    pub reserved: u32,
    pub buffer: u64,
}

pub fn handle(function: u64, _arg0: u64, _arg1: u64, _arg2: u64, _arg3: u64) -> i64 {
    match function {
        HVC_GET_VERSION => HVC_VERSION as i64,
        HVC_CONSOLE_PUTS => HVC_SUCCESS,
        HVC_RTC_GET | HVC_RTC_SET => HVC_NOT_SUPPORTED,
        HVC_RNG_READ => HVC_NOT_SUPPORTED,
        HVC_RESET | HVC_SHUTDOWN => HVC_NOT_SUPPORTED,
        HVC_BLOCK_READ | HVC_BLOCK_WRITE | HVC_BLOCK_FLUSH | HVC_BLOCK_CAPACITY => {
            HVC_NOT_SUPPORTED
        }
        _ => HVC_NOT_SUPPORTED,
    }
}
