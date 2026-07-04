use crate::error::{AppError, Result};
use crate::models::{ConnectRequest, ConnectionInfo, ProbeSummary, WireProtocol};
use probe_rs::Permissions;
use probe_rs::probe::{
    DebugProbeInfo, DebugProbeSelector, DebugProbeSelectorParseError,
    WireProtocol as ProbeWireProtocol, list::Lister,
};

pub fn list_probes() -> Result<Vec<ProbeSummary>> {
    Ok(Lister::new()
        .list_all()
        .into_iter()
        .map(probe_summary_from_info)
        .collect())
}

pub fn require_probe(identifier: Option<&str>) -> Result<String> {
    let probes = list_probes()?;

    select_probe_identifier(identifier, &probes)
}

pub fn connect_target(request: ConnectRequest) -> Result<ConnectionInfo> {
    if request.target.speed_khz == Some(0) {
        return Err(AppError::InvalidUserInput {
            detail: "烧录速度必须大于 0 kHz".to_string(),
        });
    }

    let probe_identifier = require_probe(request.probe.as_deref())?;
    let target_selector = request.target.chip.as_deref();
    let selector: DebugProbeSelector =
        probe_identifier
            .as_str()
            .try_into()
            .map_err(
                |err: DebugProbeSelectorParseError| AppError::ProbeRsFailure {
                    detail: err.to_string(),
                },
            )?;

    let mut probe = Lister::new().open(selector).map_err(probe_rs_error)?;
    let protocol = match request.target.protocol {
        WireProtocol::Swd => ProbeWireProtocol::Swd,
        WireProtocol::Jtag => ProbeWireProtocol::Jtag,
    };
    probe.select_protocol(protocol).map_err(probe_rs_error)?;

    if let Some(speed_khz) = request.target.speed_khz {
        probe.set_speed(speed_khz).map_err(probe_rs_error)?;
    }

    let session = if request.target.connect_under_reset {
        probe.attach_under_reset(target_selector, Permissions::default())
    } else {
        probe.attach(target_selector, Permissions::default())
    }
    .map_err(probe_rs_error)?;
    let chip = session.target().name.clone();

    Ok(ConnectionInfo {
        probe: probe_identifier,
        chip,
        protocol: request.target.protocol,
        speed_khz: request.target.speed_khz,
        connect_under_reset: request.target.connect_under_reset,
    })
}

fn select_probe_identifier(identifier: Option<&str>, probes: &[ProbeSummary]) -> Result<String> {
    if let Some(identifier) = identifier.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(identifier.to_string());
    }

    match probes {
        [] => Err(AppError::ProbeNotFound),
        [probe] => Ok(probe.identifier.clone()),
        _ => Err(AppError::InvalidUserInput {
            detail: "检测到多个烧录器，请手动选择一个".to_string(),
        }),
    }
}

fn probe_summary_from_info(probe: DebugProbeInfo) -> ProbeSummary {
    ProbeSummary {
        identifier: selector_string_from_info(&probe),
        vendor_id: probe.vendor_id,
        product_id: probe.product_id,
        serial_number: probe.serial_number,
        product: Some(probe.identifier),
    }
}

fn selector_string_from_info(probe: &DebugProbeInfo) -> String {
    let selector = DebugProbeSelector::from(probe);
    let mut value = format!("{:04x}:{:04x}", selector.vendor_id, selector.product_id);

    // probe-rs 0.31 parses VID:PID-interface[:serial]. Its Display impl omits
    // interface, so preserve it here to keep this identifier usable for open.
    if let Some(interface) = selector.interface {
        value.push_str(&format!("-{interface}"));
    }

    if let Some(serial_number) = selector.serial_number {
        value.push(':');
        value.push_str(&serial_number);
    }

    value
}

fn probe_rs_error(error: impl std::fmt::Display) -> AppError {
    AppError::ProbeRsFailure {
        detail: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    fn probe(identifier: &str) -> ProbeSummary {
        ProbeSummary {
            identifier: identifier.to_string(),
            vendor_id: 0x1234,
            product_id: 0x5678,
            serial_number: None,
            product: None,
        }
    }

    #[test]
    fn select_probe_identifier_should_return_explicit_trimmed_identifier() {
        let identifier =
            select_probe_identifier(Some("  probe-1  "), &[]).expect("explicit probe should win");

        assert_eq!(identifier, "probe-1");
    }

    #[test]
    fn select_probe_identifier_should_return_probe_not_found_for_empty_probes() {
        let err = select_probe_identifier(None, &[]).expect_err("empty probes should be rejected");

        assert!(matches!(err, AppError::ProbeNotFound));
    }

    #[test]
    fn select_probe_identifier_should_return_only_probe() {
        let identifier =
            select_probe_identifier(None, &[probe("only-probe")]).expect("single probe is valid");

        assert_eq!(identifier, "only-probe");
    }

    #[test]
    fn select_probe_identifier_should_reject_multiple_probes_without_explicit_identifier() {
        let err = select_probe_identifier(None, &[probe("probe-1"), probe("probe-2")])
            .expect_err("multiple probes require explicit selection");

        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }
}
