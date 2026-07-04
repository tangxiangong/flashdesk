use crate::error::{AppError, Result};
use crate::models::ProbeSummary;
use probe_rs::probe::{DebugProbeInfo, DebugProbeSelector, list::Lister};

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

fn select_probe_identifier(identifier: Option<&str>, probes: &[ProbeSummary]) -> Result<String> {
    if let Some(identifier) = identifier.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(identifier.to_string());
    }

    match probes {
        [] => Err(AppError::ProbeNotFound),
        [probe] => Ok(probe.identifier.clone()),
        _ => Err(AppError::InvalidUserInput {
            detail: "检测到多个探针，请手动选择一个探针".to_string(),
        }),
    }
}

fn probe_summary_from_info(probe: DebugProbeInfo) -> ProbeSummary {
    ProbeSummary {
        identifier: selector_string_from_info(&probe),
        vendor_id: probe.vendor_id,
        product_id: probe.product_id,
        serial_number: probe.serial_number,
        product: None,
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
