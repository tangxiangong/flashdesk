use crate::{
    error::{AppError, Result, describe_error_chain},
    models::{
        EraseRequest, JobId, JobKind, JobStage, MemoryReadResult, MemoryRequest, TargetSelection,
        WireProtocol,
    },
    services::jobs::{emit_job_event, new_job_id},
    services::probe::require_probe,
};
use probe_rs::{
    Permissions, Session,
    flashing::{FlashProgress, erase_all},
    probe::{
        DebugProbeSelector, DebugProbeSelectorParseError, WireProtocol as ProbeWireProtocol,
        list::Lister,
    },
};
use tauri::AppHandle;

const MAX_MEMORY_TRANSFER_BYTES: u32 = 4096;

/// 同步读取目标内存区域。
pub fn read_memory(request: MemoryRequest) -> Result<MemoryReadResult> {
    validate_memory_range(request.address, request.length)?;
    let mut session = open_session(&request.target, request.probe.as_deref())?;
    let mut core = session.core(0).map_err(probe_rs_error)?;
    let mut data = vec![0; request.length as usize];

    probe_rs::MemoryInterface::read(&mut core, request.address, &mut data)
        .map_err(probe_rs_error)?;

    Ok(MemoryReadResult {
        address: request.address,
        length: request.length,
        data_hex: encode_hex(&data),
    })
}

/// 创建后台整片擦除任务并返回任务 ID。
pub fn erase_target(app: &AppHandle, request: EraseRequest) -> Result<JobId> {
    let job_id = new_job_id();
    emit_job_event(
        app,
        &job_id,
        JobKind::Erase,
        JobStage::Queued,
        Some(0.0),
        "擦除任务已创建",
    )?;

    match run_erase_target(app, &job_id, request) {
        Ok(()) => Ok(job_id),
        Err(err) => {
            let _ = emit_job_event(
                app,
                &job_id,
                JobKind::Erase,
                JobStage::Failed,
                None,
                failed_message(&err),
            );
            Err(err)
        }
    }
}

fn run_erase_target(app: &AppHandle, job_id: &JobId, request: EraseRequest) -> Result<()> {
    emit_job_event(
        app,
        job_id,
        JobKind::Erase,
        JobStage::Connecting,
        Some(0.2),
        "正在连接目标芯片",
    )?;

    let mut session = open_session(&request.target, request.probe.as_deref())?;
    let mut progress = FlashProgress::empty();

    emit_job_event(
        app,
        job_id,
        JobKind::Erase,
        JobStage::Erasing,
        Some(0.5),
        "正在擦除目标 Flash",
    )?;

    erase_all(&mut session, &mut progress, false).map_err(probe_rs_error)?;

    emit_job_event(
        app,
        job_id,
        JobKind::Erase,
        JobStage::Completed,
        Some(1.0),
        "擦除完成",
    )?;

    Ok(())
}

fn open_session(target: &TargetSelection, probe_identifier: Option<&str>) -> Result<Session> {
    if target.speed_khz == Some(0) {
        return Err(AppError::InvalidUserInput {
            detail: "烧录速度必须大于 0 kHz".to_string(),
        });
    }

    let probe_identifier = require_probe(probe_identifier)?;
    let target_selector = target.chip.as_deref();
    let selector: DebugProbeSelector =
        probe_identifier
            .as_str()
            .try_into()
            .map_err(
                |err: DebugProbeSelectorParseError| AppError::ProbeRsFailure {
                    detail: describe_error_chain(&err),
                },
            )?;

    let mut probe = Lister::new().open(selector).map_err(probe_rs_error)?;
    let protocol = match target.protocol {
        WireProtocol::Swd => ProbeWireProtocol::Swd,
        WireProtocol::Jtag => ProbeWireProtocol::Jtag,
    };
    probe.select_protocol(protocol).map_err(probe_rs_error)?;

    if let Some(speed_khz) = target.speed_khz {
        probe.set_speed(speed_khz).map_err(probe_rs_error)?;
    }

    if target.connect_under_reset {
        probe.attach_under_reset(target_selector, Permissions::default())
    } else {
        probe.attach(target_selector, Permissions::default())
    }
    .map_err(probe_rs_error)
}

fn validate_memory_range(address: u64, length: u32) -> Result<()> {
    if length == 0 {
        return Err(AppError::InvalidUserInput {
            detail: "长度必须大于 0".to_string(),
        });
    }

    if length > MAX_MEMORY_TRANSFER_BYTES {
        return Err(AppError::InvalidUserInput {
            detail: "单次最多读取 4096 字节".to_string(),
        });
    }

    address
        .checked_add(u64::from(length))
        .ok_or_else(|| AppError::InvalidUserInput {
            detail: "地址范围溢出".to_string(),
        })?;

    Ok(())
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn failed_message(error: &AppError) -> String {
    let response = error.to_response();
    match response.detail {
        Some(detail) => format!("{}：{}", response.message, detail),
        None => response.message,
    }
}

fn probe_rs_error(error: impl std::error::Error + 'static) -> AppError {
    AppError::ProbeRsFailure {
        detail: describe_error_chain(&error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_memory_range_should_reject_zero_length() {
        let err = validate_memory_range(0x2000_0000, 0).unwrap_err();
        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }

    #[test]
    fn validate_memory_range_should_reject_too_large_length() {
        let err = validate_memory_range(0x2000_0000, MAX_MEMORY_TRANSFER_BYTES + 1).unwrap_err();
        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }

    #[test]
    fn encode_hex_should_render_lowercase_pairs() {
        assert_eq!(encode_hex(&[0x00, 0x0f, 0xa5, 0xff]), "000fa5ff");
    }
}
