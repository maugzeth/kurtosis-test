//!

use std::process::Command;

use crate::errors::KurtosisNetworkError;
use crate::utils;

/// Enclave service port info structure.
#[derive(Debug, Clone)]
pub struct EnclaveServicePort {
    /// Port name e.g. "http", "metrics", "rpc", etc
    pub name: String,
    /// Port protocol description e.g. "8080/tcp"
    pub protocol: String,
    /// URL to connect to service e.g. "127.0.0.1:56766".
    pub url: String,
}

impl EnclaveServicePort {
    /// Check if port is JSON-RPC port.
    pub fn is_rpc_port(&self) -> bool {
        self.name.eq("rpc")
    }

    /// Check if port is a engine RPC.
    pub fn is_engine_rpc_port(&self) -> bool {
        self.name.eq("engine-rpc")
    }
}

/// Enclave service structure.
#[derive(Debug)]
pub struct EnclaveService {
    /// Unique identifier for service
    pub uuid: String,
    /// Human readable name of service
    pub name: String,
    /// Status of the service e.g. "RUNNING"
    pub status: String,
    /// List of service ports
    pub ports: Vec<EnclaveServicePort>,
}

impl EnclaveService {
    /// Check if service is execution layer service, name is prefixed with "el-" and has RPC service port.
    pub fn is_exec_layer(&self) -> bool {
        self.name.contains("el-") && self.ports.iter().find(|port| port.is_rpc_port()).is_some()
    }
}

/// Start Kurtosis engine locally in docker using ethereum-package.
///
/// Command:
/// `kurtosis run github.com/kurtosis-tech/ethereum-package --args-file {network_params_file_name}`
///
/// Launch `ethereum-package` enclave, this also spins up engine if not already done so.
pub fn start_engine(network_params_file_name: &str) -> Result<(), KurtosisNetworkError> {
    let mut net_param_conf_path = "configs/".to_string();
    net_param_conf_path.push_str(network_params_file_name);

    let cmd_result = Command::new("kurtosis")
        .arg("run")
        .arg("github.com/kurtosis-tech/ethereum-package")
        .arg("--args-file")
        .arg(net_param_conf_path)
        .output();
    match cmd_result {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToStartKurtosisEngine);
            }
            Ok(())
        }
        Err(_) => Err(KurtosisNetworkError::FailedToStartKurtosisEngine),
    }
}

/// Check if Kurtosis CLI is installed locally.
///
/// Command:
/// `kurtosis version`
///
/// If getting version fails, we know Kurtosis is not installed, else it is.
pub fn is_cli_installed() -> Result<(), KurtosisNetworkError> {
    let cmd_result = Command::new("kurtosis").arg("version").output();
    match cmd_result {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToCheckEngineStatus);
            }
            Ok(())
        }
        Err(_) => Err(KurtosisNetworkError::CliNotInstalled),
    }
}

/// Check if Kurtosis engine is running locally in docker.
///
/// Command:
/// `kurtosis engine status`
///
/// Check if kurtosis engine is running by checking for presence of string:
///
/// `"Kurtosis engine is running with the following info"`
///
/// If present in standard output of command, return `true` if so, else `false`.
pub fn is_engine_running() -> Result<bool, KurtosisNetworkError> {
    let cmd_out = Command::new("kurtosis")
        .arg("engine")
        .arg("status")
        .output();
    match cmd_out {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToCheckEngineStatus);
            }
            let command_stdout = String::from_utf8_lossy(&out.stdout);
            Ok(command_stdout.contains("Kurtosis engine is running with the following info"))
        }
        Err(_) => Err(KurtosisNetworkError::FailedToCheckEngineStatus),
    }
}

/// Fetch all active/running services in enclave.
///
/// Command:
/// `kurtosis enclave inspect {enclave_uuid}`
///
/// Returns a list of enclave services parsed from standard output of enclave inspect.
pub fn get_running_services(
    enclave_uuid: &str,
) -> Result<Vec<EnclaveService>, KurtosisNetworkError> {
    let cmd_out = Command::new("kurtosis")
        .arg("enclave")
        .arg("inspect")
        .arg(enclave_uuid)
        .output();
    match cmd_out {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToGetEnclaveServices);
            }
            let command_stdout = String::from_utf8_lossy(&out.stdout);
            let enclave_services = utils::parse_services_from_enclave_inspect(&command_stdout.to_string());
            Ok(enclave_services)
        }
        Err(_) => Err(KurtosisNetworkError::FailedToGetEnclaveServices),
    }
}

/// Deletes an enclave.
///
/// Command:
/// `kurtosis enclave rm {enclave_uuid} --force`
///
/// We force removal using `--force` to prevent having to stop enclave, then removing.
/// Instead this does it all within a single command.
pub fn delete_enclave(enclave_uuid: &str) -> Result<(), KurtosisNetworkError> {
    let cmd_out = Command::new("kurtosis")
        .arg("enclave")
        .arg("rm")
        .arg(enclave_uuid)
        .arg("--force")
        .output();
    match cmd_out {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToDestroyEnclave);
            }
            Ok(())
        }
        Err(_) => Err(KurtosisNetworkError::FailedToDestroyEnclave),
    }
}
