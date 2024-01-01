//!

use regex::Regex;

use crate::{EnclaveService, EnclaveServicePortInfo};

/// Parses raw services output from kurtosis enclave inspect command output to [`EnclaveService`] type.
///
/// Example of all service line edge cases handled:
///
/// ```text
/// 0: ========================================== User Services ==========================================
/// 1: 7d28bc07285f   beacon-metrics-gazer                             http: 8080/tcp -> http://127.0.0.1:56766      RUNNING
/// 2: 93e319e73408   cl-1-lighthouse-reth                             http: 4000/tcp -> http://127.0.0.1:56741      RUNNING
/// 3:                                                                 metrics: 5054/tcp -> http://127.0.0.1:56742
/// 4: cd490f70070c   blob-spammer                                     <none>                                        RUNNING
/// ````
///
/// Parse normal service lines (lines 1, 2), some services have multiple ports (line 3) or no ports (line 4).
pub fn parse_services_from_enclave_inspect(raw_output: &String) -> Vec<EnclaveService> {
    let none_port_service_line_re =
        Regex::new(r"^([a-f0-9]{12})\s+(\S+)\s+(<none>)\s+(\S+)").unwrap();
    let continue_service_line_re = Regex::new(r"^\s+(\S+)(:)\s+(\S+)\s+(\S+)\s+(\S+)\s+").unwrap();
    let new_service_line_re =
        Regex::new(r"^([a-f0-9]{12})\s+(\S+)\s+(\S+)(:)\s(\d+\S+)\s(->)\s(\S+)(\s+)(\S+)$")
            .unwrap();

    let mut services: Vec<EnclaveService> = Vec::new();
    raw_output.split("\n").for_each(|line| {
        // DEBUG: println!("{:?}", line);

        // if we match a new service line, return new enclave service entry
        if let Some(caps) = new_service_line_re.captures(line) {
            let port_info = EnclaveServicePortInfo {
                name: caps.get(3).unwrap().as_str().to_string(),
                protocol: caps.get(5).unwrap().as_str().to_string(),
                url: remove_http_from_url(caps.get(7).unwrap().as_str().to_string()),
            };
            services.push(EnclaveService {
                uuid: caps.get(1).unwrap().as_str().to_string(),
                name: caps.get(2).unwrap().as_str().to_string(),
                status: caps.get(9).unwrap().as_str().to_string(),
                ports: vec![port_info],
            });
            return;
        }

        // if we match a none port service, return new enclave service entry with no ports
        if let Some(caps) = none_port_service_line_re.captures(line) {
            services.push(EnclaveService {
                uuid: caps.get(1).unwrap().as_str().to_string(),
                name: caps.get(2).unwrap().as_str().to_string(),
                status: caps.get(4).unwrap().as_str().to_string(),
                ports: vec![],
            });
            return;
        }

        // if we match a continued service port line, update last service by appending to ports
        if let Some(caps) = continue_service_line_re.captures(line) {
            let mut last_service = services.pop().unwrap();
            let mut updated_service_ports = last_service.ports;
            updated_service_ports.push(EnclaveServicePortInfo {
                name: caps.get(1).unwrap().as_str().to_string(),
                protocol: caps.get(3).unwrap().as_str().to_string(),
                url: remove_http_from_url(caps.get(5).unwrap().as_str().to_string()),
            });
            last_service.ports = updated_service_ports;
            services.push(last_service);
            return;
        }
    });

    services
}

/// Removes "https://" prefix from url or returns original no prefix found.
fn remove_http_from_url(url: String) -> String {
    if url.contains("http://") {
        url.replace("http://", "")
    } else {
        url
    }
}