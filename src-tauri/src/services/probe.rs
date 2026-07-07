use crate::{
    error::{AppError, Result},
    models::{ConnectRequest, ConnectionInfo, ProbeSummary, TargetCandidate, WireProtocol},
    services::target::arm_target_candidates,
};
use probe_rs::{
    Permissions,
    architecture::arm::{
        ArmChipInfo, dp::DpAddress, memory::romtable::Component, sequences::DefaultArmSequence,
    },
    config::{Registry, RegistryError},
    probe::{
        DebugProbeInfo, DebugProbeSelector, DebugProbeSelectorParseError,
        WireProtocol as ProbeWireProtocol, list::Lister,
    },
};

/// 列出当前系统可见的调试探针。
pub fn list_probes() -> Result<Vec<ProbeSummary>> {
    Ok(Lister::new()
        .list_all()
        .into_iter()
        .map(probe_summary_from_info)
        .collect())
}

/// 解析前端探针选择；为空时要求系统中只有一个探针。
pub fn require_probe(identifier: Option<&str>) -> Result<String> {
    let probes = list_probes()?;

    select_probe_identifier(identifier, &probes)
}

/// 打开探针会话以验证目标连接参数。
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

    let session_result = if request.target.connect_under_reset {
        probe.attach_under_reset(target_selector, Permissions::default())
    } else {
        probe.attach(target_selector, Permissions::default())
    };
    let session = match session_result {
        Ok(session) => session,
        Err(error) if target_selector.is_none() && is_chip_autodetect_failure(&error) => {
            let candidates = identify_target_candidates(
                &probe_identifier,
                protocol,
                request.target.speed_khz,
                request.target.connect_under_reset,
            );
            return Err(AppError::TargetIdentifyFailed {
                detail: target_identify_detail(candidates.as_ref(), &error),
                candidates: candidates.unwrap_or_default(),
            });
        }
        Err(error) => return Err(probe_rs_error(error)),
    };
    let chip = session.target().name.clone();

    Ok(ConnectionInfo {
        probe: probe_identifier,
        chip,
        protocol: request.target.protocol,
        speed_khz: request.target.speed_khz,
        connect_under_reset: request.target.connect_under_reset,
    })
}

fn identify_target_candidates(
    probe_identifier: &str,
    protocol: ProbeWireProtocol,
    speed_khz: Option<u32>,
    connect_under_reset: bool,
) -> std::result::Result<Vec<TargetCandidate>, String> {
    let selector: DebugProbeSelector = probe_identifier
        .try_into()
        .map_err(|err: DebugProbeSelectorParseError| err.to_string())?;
    let mut probe = Lister::new()
        .open(selector)
        .map_err(|err| err.to_string())?;
    probe
        .select_protocol(protocol)
        .map_err(|err| err.to_string())?;

    if let Some(speed_khz) = speed_khz {
        probe.set_speed(speed_khz).map_err(|err| err.to_string())?;
    }

    if connect_under_reset {
        probe
            .attach_to_unspecified_under_reset()
            .map_err(|err| err.to_string())?;
    } else {
        probe
            .attach_to_unspecified()
            .map_err(|err| err.to_string())?;
    }

    let Some(chip_info) = read_arm_chip_info(probe).map_err(|err| err.to_string())? else {
        return Ok(Vec::new());
    };

    let registry = Registry::from_builtin_families();
    Ok(arm_target_candidates(&registry, chip_info))
}

fn read_arm_chip_info(
    probe: probe_rs::probe::Probe,
) -> std::result::Result<Option<ArmChipInfo>, probe_rs::Error> {
    if !probe.has_arm_debug_interface() {
        return Ok(None);
    }

    let interface = match probe.try_into_arm_debug_interface(DefaultArmSequence::create()) {
        Ok(interface) => interface,
        Err((_returned_probe, error)) => return Err(error.into()),
    };

    let mut interface = interface;
    let dp_address = DpAddress::Default;
    let mut found_chip_info = None;
    for ap in interface.access_ports(dp_address)? {
        if let Ok(mut memory) = interface.memory_interface(&ap) {
            let base_address = memory.base_address()?;
            let component = Component::try_parse(&mut *memory, base_address)
                .map_err(|err| probe_rs::Error::Other(err.to_string()))?;

            if let Component::Class1RomTable(component_id, _) = component
                && let Some(jep106) = component_id.peripheral_id().jep106()
            {
                found_chip_info = Some(ArmChipInfo {
                    manufacturer: jep106,
                    part: component_id.peripheral_id().part(),
                });
                break;
            }
        }
    }

    let _probe = interface.close();
    Ok(found_chip_info)
}

fn is_chip_autodetect_failure(error: &probe_rs::Error) -> bool {
    matches!(
        error,
        probe_rs::Error::ChipNotFound(RegistryError::ChipAutodetectFailed)
    )
}

fn target_identify_detail(
    candidates: std::result::Result<&Vec<TargetCandidate>, &String>,
    error: &probe_rs::Error,
) -> String {
    match candidates {
        Ok(candidates) if !candidates.is_empty() => {
            format!(
                "自动识别未能唯一确定目标，已缩小到 {} 个候选",
                candidates.len()
            )
        }
        Ok(_) => format!("自动识别未能确定目标，且当前探测信息没有匹配候选：{error}"),
        Err(detail) => format!("自动识别失败，候选探测也失败：{detail}"),
    }
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
