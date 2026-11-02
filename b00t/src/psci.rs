pub const PSCI_VERSION: u32 = 0x8400_0000;
pub const PSCI_CPU_SUSPEND: u32 = 0x8400_0001;
pub const PSCI_CPU_OFF: u32 = 0x8400_0002;
pub const PSCI_CPU_ON: u32 = 0x8400_0003;
pub const PSCI_AFFINITY_INFO: u32 = 0x8400_0004;
pub const PSCI_MIGRATE: u32 = 0x8400_0005;
pub const PSCI_SYSTEM_OFF: u32 = 0x8400_0008;
pub const PSCI_SYSTEM_RESET: u32 = 0x8400_0009;
pub const PSCI_FEATURES: u32 = 0x8400_000a;

pub const PSCI_VERSION_1_1: u32 = 0x0001_0001;

pub const PSCI_SUCCESS: i64 = 0;
pub const PSCI_NOT_SUPPORTED: i64 = -1;
pub const PSCI_INVALID_PARAMETERS: i64 = -2;
pub const PSCI_ALREADY_ON: i64 = -3;
pub const PSCI_DENIED: i64 = -4;
pub const PSCI_INVALID_ADDRESS: i64 = -5;
pub const PSCI_INTERNAL_FAILURE: i64 = -6;
pub const PSCI_NOT_PRESENT: i64 = -7;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CpuState {
    Off,
    On,
    Suspended,
}

#[derive(Clone, Copy)]
pub struct CpuContext {
    pub mpidr: u64,
    pub state: CpuState,
    pub entry_point: u64,
    pub context_id: u64,
}

pub const MAX_CPUS: usize = 16;

pub struct PsciState {
    pub cpus: [CpuContext; MAX_CPUS],
    pub cpu_count: usize,
}

impl PsciState {
    pub fn new() -> Self {
        let mut state = PsciState {
            cpus: [CpuContext {
                mpidr: 0,
                state: CpuState::Off,
                entry_point: 0,
                context_id: 0,
            }; MAX_CPUS],
            cpu_count: 0,
        };
        state.cpus[0].mpidr = 0x8000_0000;
        state.cpus[0].state = CpuState::On;
        state.cpu_count = 1;
        state
    }

    pub fn find_cpu(&mut self, mpidr: u64) -> Option<usize> {
        for index in 0..self.cpu_count {
            if self.cpus[index].mpidr == mpidr {
                return Some(index);
            }
        }
        if self.cpu_count < MAX_CPUS {
            let index = self.cpu_count;
            self.cpus[index].mpidr = mpidr;
            self.cpus[index].state = CpuState::Off;
            self.cpu_count += 1;
            return Some(index);
        }
        None
    }

    pub fn cpu_on(&mut self, target_cpu: u64, entry_point: u64, context_id: u64) -> i64 {
        if entry_point == 0 {
            return PSCI_INVALID_ADDRESS;
        }

        let Some(index) = self.find_cpu(target_cpu) else {
            return PSCI_INVALID_PARAMETERS;
        };

        if self.cpus[index].state == CpuState::On {
            return PSCI_ALREADY_ON;
        }

        self.cpus[index].state = CpuState::On;
        self.cpus[index].entry_point = entry_point;
        self.cpus[index].context_id = context_id;
        PSCI_SUCCESS
    }
}

pub fn handle(state: &mut PsciState, function: u32, args: [u64; 4]) -> i64 {
    match function {
        PSCI_VERSION => PSCI_VERSION_1_1 as i64,
        PSCI_FEATURES => match args[0] as u32 {
            PSCI_CPU_ON | PSCI_CPU_OFF | PSCI_CPU_SUSPEND | PSCI_AFFINITY_INFO
            | PSCI_SYSTEM_OFF | PSCI_SYSTEM_RESET => PSCI_SUCCESS,
            _ => PSCI_NOT_SUPPORTED,
        },
        PSCI_CPU_ON => state.cpu_on(args[0], args[1], args[2]),
        PSCI_CPU_OFF => {
            // The calling CPU would be marked off by the caller context.
            PSCI_SUCCESS
        }
        PSCI_CPU_SUSPEND => PSCI_NOT_SUPPORTED,
        PSCI_AFFINITY_INFO => PSCI_SUCCESS,
        PSCI_SYSTEM_OFF => PSCI_NOT_SUPPORTED,
        PSCI_SYSTEM_RESET => PSCI_NOT_SUPPORTED,
        _ => PSCI_NOT_SUPPORTED,
    }
}
