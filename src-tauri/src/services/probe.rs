use crate::{
    error::{AppError, Result},
    models::{
        ConnectRequest, ConnectionInfo, ProbeSummary, TargetCandidate, TargetInformation,
        WireProtocol,
    },
    services::target::arm_target_candidates,
};
use probe_rs::{
    MemoryInterface, Permissions,
    architecture::arm::{
        ArmChipInfo, dp::DpAddress, memory::ArmMemoryInterface, memory::romtable::Component,
        sequences::DefaultArmSequence,
    },
    config::{Registry, RegistryError},
    probe::{
        DebugProbeInfo, DebugProbeSelector, DebugProbeSelectorParseError,
        WireProtocol as ProbeWireProtocol, list::Lister,
    },
};

const CPUID_ADDRESS: u64 = 0xE000_ED00;
const STM32_DBGMCU_IDCODE_ADDRESSES: [u64; 5] = [
    0xE004_2000,
    0x4001_5800,
    0xE004_4000,
    0x5C00_1000,
    0xE00E_1000,
];

struct TargetIdentification {
    candidates: Vec<TargetCandidate>,
    target_information: Option<TargetInformation>,
}

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
    let selector: DebugProbeSelector = probe_identifier
        .as_str()
        .try_into()
        .map_err(|err: DebugProbeSelectorParseError| AppError::operation(err))?;

    let mut probe = Lister::new().open(selector).map_err(AppError::operation)?;
    let protocol = match request.target.protocol {
        WireProtocol::Swd => ProbeWireProtocol::Swd,
        WireProtocol::Jtag => ProbeWireProtocol::Jtag,
    };
    probe
        .select_protocol(protocol)
        .map_err(AppError::operation)?;

    if let Some(speed_khz) = request.target.speed_khz {
        probe.set_speed(speed_khz).map_err(AppError::operation)?;
    }

    let session_result = if request.target.connect_under_reset {
        probe.attach_under_reset(target_selector, Permissions::default())
    } else {
        probe.attach(target_selector, Permissions::default())
    };
    let session = match session_result {
        Ok(session) => session,
        Err(error) if target_selector.is_none() && is_chip_autodetect_failure(&error) => {
            let identification = identify_target_candidates(
                &probe_identifier,
                protocol,
                request.target.speed_khz,
                request.target.connect_under_reset,
            );
            return Err(AppError::TargetIdentifyFailed {
                detail: target_identify_detail(identification.as_ref(), &error),
                candidates: identification
                    .as_ref()
                    .map(|value| value.candidates.clone())
                    .unwrap_or_default(),
                target_information: identification
                    .ok()
                    .and_then(|value| value.target_information),
            });
        }
        Err(error) => return Err(AppError::operation(error)),
    };
    let mut session = session;
    let chip = session.target().name.clone();
    let target_information = if chip.starts_with("STM32") {
        session
            .core(0)
            .ok()
            .map(|mut core| target_information_from_core(&mut core, "STM32"))
    } else {
        None
    };

    Ok(ConnectionInfo {
        probe: probe_identifier,
        chip,
        protocol: request.target.protocol,
        speed_khz: request.target.speed_khz,
        connect_under_reset: request.target.connect_under_reset,
        target_information,
    })
}

fn identify_target_candidates(
    probe_identifier: &str,
    protocol: ProbeWireProtocol,
    speed_khz: Option<u32>,
    connect_under_reset: bool,
) -> std::result::Result<TargetIdentification, String> {
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

    let Some((chip_info, target_information)) =
        read_arm_chip_info(probe).map_err(|err| err.to_string())?
    else {
        return Ok(TargetIdentification {
            candidates: Vec::new(),
            target_information: None,
        });
    };

    let registry = Registry::from_builtin_families();
    Ok(TargetIdentification {
        candidates: arm_target_candidates(&registry, chip_info),
        target_information: Some(target_information),
    })
}

fn read_arm_chip_info(
    probe: probe_rs::probe::Probe,
) -> std::result::Result<Option<(ArmChipInfo, TargetInformation)>, probe_rs::Error> {
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
                let manufacturer = jep106.get().unwrap_or("ARM");
                let chip_info = ArmChipInfo {
                    manufacturer: jep106,
                    part: component_id.peripheral_id().part(),
                };
                let device_type = if manufacturer == "STMicroelectronics" {
                    "STM32"
                } else {
                    manufacturer
                };
                found_chip_info = Some((
                    chip_info,
                    target_information_from_arm_memory(&mut *memory, device_type),
                ));
                break;
            }
        }
    }

    let _probe = interface.close();
    Ok(found_chip_info)
}

fn target_information_from_arm_memory(
    memory: &mut dyn ArmMemoryInterface,
    device_type: &str,
) -> TargetInformation {
    let cpuid = memory.read_word_32(CPUID_ADDRESS).ok();
    let idcode = (device_type == "STM32")
        .then(|| read_stm32_idcode_arm(memory))
        .flatten();

    TargetInformation {
        device_type: device_type.to_string(),
        device_id: idcode.map(|value| (value & 0x0fff) as u16),
        revision_id: idcode.map(|value| (value >> 16) as u16),
        cpu: cpuid.and_then(cpu_name_from_cpuid).map(str::to_string),
    }
}

fn target_information_from_core(
    core: &mut probe_rs::Core<'_>,
    device_type: &str,
) -> TargetInformation {
    let cpuid = core.read_word_32(CPUID_ADDRESS).ok();
    let idcode = (device_type == "STM32")
        .then(|| read_stm32_idcode_core(core))
        .flatten();

    TargetInformation {
        device_type: device_type.to_string(),
        device_id: idcode.map(|value| (value & 0x0fff) as u16),
        revision_id: idcode.map(|value| (value >> 16) as u16),
        cpu: cpuid.and_then(cpu_name_from_cpuid).map(str::to_string),
    }
}

fn read_stm32_idcode_arm(memory: &mut dyn ArmMemoryInterface) -> Option<u32> {
    STM32_DBGMCU_IDCODE_ADDRESSES.iter().find_map(|address| {
        let value = memory.read_word_32(*address).ok()?;
        let device_id = value & 0x0fff;
        (value != 0 && value != u32::MAX && device_id != 0 && device_id != 0x0fff).then_some(value)
    })
}

fn read_stm32_idcode_core(core: &mut probe_rs::Core<'_>) -> Option<u32> {
    STM32_DBGMCU_IDCODE_ADDRESSES.iter().find_map(|address| {
        let value = core.read_word_32(*address).ok()?;
        let device_id = value & 0x0fff;
        (value != 0 && value != u32::MAX && device_id != 0 && device_id != 0x0fff).then_some(value)
    })
}

fn cpu_name_from_cpuid(cpuid: u32) -> Option<&'static str> {
    match (cpuid >> 4) & 0x0fff {
        0xC20 => Some("Cortex-M0"),
        0xC60 => Some("Cortex-M0+"),
        0xC23 => Some("Cortex-M3"),
        0xC24 => Some("Cortex-M4"),
        0xC27 => Some("Cortex-M7"),
        0xD20 => Some("Cortex-M23"),
        0xD21 => Some("Cortex-M33"),
        0xD22 => Some("Cortex-M55"),
        0xD23 => Some("Cortex-M85"),
        _ => None,
    }
}

fn is_chip_autodetect_failure(error: &probe_rs::Error) -> bool {
    matches!(
        error,
        probe_rs::Error::ChipNotFound(RegistryError::ChipAutodetectFailed)
    )
}

fn target_identify_detail(
    identification: std::result::Result<&TargetIdentification, &String>,
    error: &probe_rs::Error,
) -> String {
    match identification {
        Ok(identification) if !identification.candidates.is_empty() => {
            format!(
                "自动识别未能唯一确定目标，已缩小到 {} 个候选",
                identification.candidates.len()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    #[derive(Debug, thiserror::Error)]
    #[error("probe summary")]
    struct ProbeSummaryError {
        #[source]
        source: std::io::Error,
    }

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

    #[test]
    fn cpu_name_from_cpuid_should_identify_cortex_m3() {
        let name = cpu_name_from_cpuid(0x411F_C231);

        assert_eq!(name, Some("Cortex-M3"));
    }

    #[test]
    fn cpu_name_from_cpuid_should_ignore_unknown_part_number() {
        let name = cpu_name_from_cpuid(0x410F_FFF0);

        assert_eq!(name, None);
    }

    #[test]
    fn operation_error_should_preserve_chinese_summary_and_root_source_message() {
        let error = AppError::operation(ProbeSummaryError {
            source: std::io::Error::other("USB transfer failed"),
        });
        let response = error.to_response();

        assert_eq!(
            (response.message.as_str(), response.detail.as_deref()),
            ("操作失败", Some("USB transfer failed"))
        );
    }
}
