pub mod system {
    use alloc::string::String;
    use crate::system::system::CallType::{Undefined, FileManipulation, ProcessControl, DeviceManipulation, InformationMaintenance, Communication};

    pub enum ProcessControlCall {
        // TODO
    }
    pub enum FileManipulationCall {
        // TODO
    }
    pub enum DeviceManipulationCall {
        // TODO
    }
    pub enum InformationMaintenanceCall {
        // TODO
    }
    pub enum CommunicationCall {
        // TODO
    }

    pub enum CallType {
        ProcessControl(ProcessControlCall), FileManipulation(FileManipulationCall),
        DeviceManipulation(DeviceManipulationCall), InformationMaintenance(InformationMaintenanceCall),
        Communication(CommunicationCall), Undefined()
    }

    pub fn syscall(t: CallType) {
        match t {
            ProcessControl(pcc) => {
                // TODO
            }
            FileManipulation(fmc) => {
                // TODO
            }
            DeviceManipulation(dmc) => {
                // TODO
            }
            InformationMaintenance(imc) => {
                // TODO
            }
            Communication(dmc) => {
                // TODO
            }
            Undefined() => {
                // TODO
            }
        }
    }
}