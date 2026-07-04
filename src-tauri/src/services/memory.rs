use crate::error::{AppError, Result};
use crate::models::{
    DumpMemoryRequest, EraseRequest, JobId, JobKind, JobStage, MemoryReadResult, MemoryRequest,
    TargetActionRequest, TargetSelection, TargetStatus, WireProtocol, WriteMemoryRequest,
};
use crate::services::jobs::{emit_job_event, new_job_id};
use crate::services::probe::require_probe;
use crate::services::target::require_chip;
use probe_rs::flashing::{FlashProgress, erase, erase_all};
use probe_rs::probe::{
    DebugProbeSelector, DebugProbeSelectorParseError, WireProtocol as ProbeWireProtocol,
    list::Lister,
};
use probe_rs::{Permissions, Session};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tauri::AppHandle;

const MAX_MEMORY_TRANSFER_BYTES: u32 = 1024 * 1024;

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

pub fn write_memory(app: &AppHandle, request: WriteMemoryRequest) -> Result<JobId> {
    let bytes = parse_hex_bytes(&request.data_hex)?;
    validate_memory_range(request.address, bytes.len() as u32)?;
    let job_id = new_job_id();
    emit_job_event(
        app,
        &job_id,
        JobKind::WriteMemory,
        JobStage::Queued,
        Some(0.0),
        "内存写入任务已创建",
    )?;

    match run_write_memory(app, &job_id, request, bytes) {
        Ok(()) => Ok(job_id),
        Err(err) => {
            let _ = emit_job_event(
                app,
                &job_id,
                JobKind::WriteMemory,
                JobStage::Failed,
                None,
                failed_message(&err),
            );
            Err(err)
        }
    }
}

pub fn dump_memory(app: &AppHandle, request: DumpMemoryRequest) -> Result<JobId> {
    validate_memory_range(request.address, request.length)?;
    if request.output_path.trim().is_empty() {
        return Err(AppError::InvalidUserInput {
            detail: "导出路径不能为空".to_string(),
        });
    }

    let job_id = new_job_id();
    emit_job_event(
        app,
        &job_id,
        JobKind::DumpMemory,
        JobStage::Queued,
        Some(0.0),
        "内存导出任务已创建",
    )?;

    match run_dump_memory(app, &job_id, request) {
        Ok(()) => Ok(job_id),
        Err(err) => {
            let _ = emit_job_event(
                app,
                &job_id,
                JobKind::DumpMemory,
                JobStage::Failed,
                None,
                failed_message(&err),
            );
            Err(err)
        }
    }
}

pub fn erase_target(app: &AppHandle, request: EraseRequest) -> Result<JobId> {
    if let Some(range) = &request.range {
        validate_erase_range(range.start, range.end)?;
    }

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

pub fn reset_target(app: &AppHandle, request: TargetActionRequest) -> Result<JobId> {
    let job_id = new_job_id();
    emit_job_event(
        app,
        &job_id,
        JobKind::Reset,
        JobStage::Queued,
        Some(0.0),
        "复位任务已创建",
    )?;

    match run_reset_target(app, &job_id, request) {
        Ok(()) => Ok(job_id),
        Err(err) => {
            let _ = emit_job_event(
                app,
                &job_id,
                JobKind::Reset,
                JobStage::Failed,
                None,
                failed_message(&err),
            );
            Err(err)
        }
    }
}

pub fn attach_target(request: TargetActionRequest) -> Result<TargetStatus> {
    let chip = require_chip(request.target.chip.as_deref())?;
    let mut session = open_session(&request.target, request.probe.as_deref())?;
    let mut core = session.core(0).map_err(probe_rs_error)?;
    let halted = core.core_halted().map_err(probe_rs_error)?;

    Ok(TargetStatus {
        chip,
        core: 0,
        halted,
    })
}

fn run_write_memory(
    app: &AppHandle,
    job_id: &JobId,
    request: WriteMemoryRequest,
    bytes: Vec<u8>,
) -> Result<()> {
    emit_job_event(
        app,
        job_id,
        JobKind::WriteMemory,
        JobStage::Connecting,
        Some(0.2),
        "正在连接目标芯片",
    )?;

    let mut session = open_session(&request.target, request.probe.as_deref())?;
    let mut core = session.core(0).map_err(probe_rs_error)?;

    emit_job_event(
        app,
        job_id,
        JobKind::WriteMemory,
        JobStage::Programming,
        Some(0.5),
        "正在写入目标内存",
    )?;

    probe_rs::MemoryInterface::write(&mut core, request.address, &bytes).map_err(probe_rs_error)?;

    emit_job_event(
        app,
        job_id,
        JobKind::WriteMemory,
        JobStage::Completed,
        Some(1.0),
        "内存写入完成",
    )?;

    Ok(())
}

fn run_dump_memory(app: &AppHandle, job_id: &JobId, request: DumpMemoryRequest) -> Result<()> {
    emit_job_event(
        app,
        job_id,
        JobKind::DumpMemory,
        JobStage::Connecting,
        Some(0.2),
        "正在连接目标芯片",
    )?;

    let mut session = open_session(&request.target, request.probe.as_deref())?;
    let mut core = session.core(0).map_err(probe_rs_error)?;
    let mut data = vec![0; request.length as usize];

    emit_job_event(
        app,
        job_id,
        JobKind::DumpMemory,
        JobStage::Programming,
        Some(0.5),
        "正在读取目标内存",
    )?;

    probe_rs::MemoryInterface::read(&mut core, request.address, &mut data)
        .map_err(probe_rs_error)?;

    fs::write(Path::new(&request.output_path), data)?;

    emit_job_event(
        app,
        job_id,
        JobKind::DumpMemory,
        JobStage::Completed,
        Some(1.0),
        "内存导出完成",
    )?;

    Ok(())
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

    if let Some(range) = request.range {
        erase(&mut session, &mut progress, range.start, range.end, false)
    } else {
        erase_all(&mut session, &mut progress, false)
    }
    .map_err(probe_rs_error)?;

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

fn run_reset_target(app: &AppHandle, job_id: &JobId, request: TargetActionRequest) -> Result<()> {
    emit_job_event(
        app,
        job_id,
        JobKind::Reset,
        JobStage::Connecting,
        Some(0.2),
        "正在连接目标芯片",
    )?;

    let mut session = open_session(&request.target, request.probe.as_deref())?;
    let mut core = session.core(0).map_err(probe_rs_error)?;

    emit_job_event(
        app,
        job_id,
        JobKind::Reset,
        JobStage::Resetting,
        Some(0.6),
        "正在复位目标芯片",
    )?;

    if request.halt_after_reset {
        core.reset_and_halt(Duration::from_millis(500))
            .map(|_| ())
            .map_err(probe_rs_error)?;
    } else {
        core.reset().map_err(probe_rs_error)?;
    }

    emit_job_event(
        app,
        job_id,
        JobKind::Reset,
        JobStage::Completed,
        Some(1.0),
        "复位完成",
    )?;

    Ok(())
}

fn open_session(target: &TargetSelection, probe_identifier: Option<&str>) -> Result<Session> {
    if target.speed_khz == Some(0) {
        return Err(AppError::InvalidUserInput {
            detail: "探针通信速度必须大于 0 kHz".to_string(),
        });
    }

    let probe_identifier = require_probe(probe_identifier)?;
    let chip = require_chip(target.chip.as_deref())?;
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
    let protocol = match target.protocol {
        WireProtocol::Swd => ProbeWireProtocol::Swd,
        WireProtocol::Jtag => ProbeWireProtocol::Jtag,
    };
    probe.select_protocol(protocol).map_err(probe_rs_error)?;

    if let Some(speed_khz) = target.speed_khz {
        probe.set_speed(speed_khz).map_err(probe_rs_error)?;
    }

    if target.connect_under_reset {
        probe.attach_under_reset(chip, Permissions::default())
    } else {
        probe.attach(chip, Permissions::default())
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
            detail: "单次内存操作最大 1 MiB".to_string(),
        });
    }

    address
        .checked_add(u64::from(length))
        .ok_or_else(|| AppError::InvalidUserInput {
            detail: "地址范围溢出".to_string(),
        })?;

    Ok(())
}

fn validate_erase_range(start: u64, end: u64) -> Result<()> {
    if start >= end {
        return Err(AppError::InvalidUserInput {
            detail: "擦除结束地址必须大于起始地址".to_string(),
        });
    }

    validate_memory_range(start, (end - start).try_into().unwrap_or(u32::MAX))
}

pub fn parse_hex_bytes(input: &str) -> Result<Vec<u8>> {
    let compact: String = input.chars().filter(|ch| !ch.is_whitespace()).collect();

    if compact.is_empty() {
        return Err(AppError::InvalidUserInput {
            detail: "十六进制数据不能为空".to_string(),
        });
    }

    if !compact.len().is_multiple_of(2) {
        return Err(AppError::InvalidUserInput {
            detail: "十六进制数据长度必须为偶数".to_string(),
        });
    }

    compact
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_value(pair[0])?;
            let low = hex_value(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_value(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(AppError::InvalidUserInput {
            detail: "十六进制数据包含非法字符".to_string(),
        }),
    }
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

fn probe_rs_error(error: impl std::fmt::Display) -> AppError {
    AppError::ProbeRsFailure {
        detail: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_bytes_should_accept_spaces() {
        let bytes = parse_hex_bytes("01 0a ff").unwrap();
        assert_eq!(bytes, vec![0x01, 0x0a, 0xff]);
    }

    #[test]
    fn parse_hex_bytes_should_reject_odd_length() {
        let err = parse_hex_bytes("abc").unwrap_err();
        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }

    #[test]
    fn validate_memory_range_should_reject_zero_length() {
        let err = validate_memory_range(0x2000_0000, 0).unwrap_err();
        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }

    #[test]
    fn validate_erase_range_should_reject_reversed_range() {
        let err = validate_erase_range(0x1000, 0x1000).unwrap_err();
        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }
}
