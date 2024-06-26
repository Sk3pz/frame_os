use crate::system::CallType::{Communication, DeviceManipulation, FileManipulation, InformationMaintenance, ProcessControl, Undefined};

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

pub fn syscall(t: CallType) { // TODO: when implemented, add a return value
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
        Communication(cc) => {
            // TODO
        }
        Undefined() => {
            // TODO
        }
    }
}