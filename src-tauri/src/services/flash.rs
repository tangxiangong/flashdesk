use crate::error::{AppError, Result};
use crate::models::{FirmwareFormat, FlashRequest, JobId, JobKind, JobStage, WireProtocol};
use crate::services::firmware::validate_firmware;
use crate::services::jobs::{emit_job_event, new_job_id};
use crate::services::probe::require_probe;
use crate::services::target::require_chip;
use std::path::Path;
use tauri::AppHandle;

pub fn validate_flash_request(request: &FlashRequest) -> Result<FirmwareFormat> {
    if request.options.reset_after && request.options.halt_after {
        return Err(AppError::InvalidUserInput {
            detail: "reset_after 和 halt_after 不能同时启用".to_string(),
        });
    }

    if request.options.skip_erase && request.options.allow_erase_all {
        return Err(AppError::InvalidUserInput {
            detail: "skip_erase 和 allow_erase_all 不能同时启用".to_string(),
        });
    }

    if request.target.speed_khz == Some(0) {
        return Err(AppError::InvalidUserInput {
            detail: "调试速度必须大于 0 kHz".to_string(),
        });
    }

    let format = validate_firmware(&request.firmware)?;
    require_chip(request.target.chip.as_deref())?;

    Ok(format)
}

fn failed_message(error: &AppError) -> String {
    let response = error.to_response();
    match response.detail {
        Some(detail) => format!("{}：{}", response.message, detail),
        None => response.message,
    }
}

pub fn flash_firmware(app: &AppHandle, request: FlashRequest) -> Result<JobId> {
    let job_id = new_job_id();

    if let Err(err) = emit_job_event(
        app,
        &job_id,
        JobKind::Flash,
        JobStage::Queued,
        Some(0.0),
        "烧录任务已创建",
    ) {
        let _ = emit_job_event(
            app,
            &job_id,
            JobKind::Flash,
            JobStage::Failed,
            None,
            failed_message(&err),
        );
        return Err(err);
    }

    match run_flash_job(app, &job_id, request) {
        Ok(()) => Ok(job_id),
        Err(err) => {
            let _ = emit_job_event(
                app,
                &job_id,
                JobKind::Flash,
                JobStage::Failed,
                None,
                failed_message(&err),
            );
            Err(err)
        }
    }
}

fn run_flash_job(app: &AppHandle, job_id: &JobId, request: FlashRequest) -> Result<()> {
    let format = validate_flash_request(&request)?;

    emit_job_event(
        app,
        job_id,
        JobKind::Flash,
        JobStage::Preparing,
        Some(0.1),
        format!("固件格式已确认：{format:?}"),
    )?;

    let firmware_path = Path::new(&request.firmware.path);
    if !firmware_path.try_exists()? {
        return Err(AppError::InvalidUserInput {
            detail: "固件文件不存在或不可访问".to_string(),
        });
    }

    run_probe_rs_flash(app, job_id, &request, format)?;

    emit_job_event(
        app,
        job_id,
        JobKind::Flash,
        JobStage::Completed,
        Some(1.0),
        "烧录完成",
    )?;

    Ok(())
}

fn run_probe_rs_flash(
    app: &AppHandle,
    job_id: &JobId,
    request: &FlashRequest,
    firmware_format: FirmwareFormat,
) -> Result<()> {
    use probe_rs::Permissions;
    use probe_rs::flashing::{DownloadOptions, download_file_with_options};
    use probe_rs::probe::{
        DebugProbeSelector, DebugProbeSelectorParseError, WireProtocol as ProbeWireProtocol,
        list::Lister,
    };
    use std::time::Duration;

    emit_job_event(
        app,
        job_id,
        JobKind::Flash,
        JobStage::Connecting,
        Some(0.2),
        "正在连接调试探针",
    )?;

    let probe_identifier = require_probe(request.probe.as_deref())?;
    let chip = require_chip(request.target.chip.as_deref())?;
    let selector: DebugProbeSelector =
        probe_identifier
            .as_str()
            .try_into()
            .map_err(
                |err: DebugProbeSelectorParseError| AppError::ProbeRsFailure {
                    detail: err.to_string(),
                },
            )?;

    let mut probe = Lister::new()
        .open(selector)
        .map_err(|err| AppError::ProbeRsFailure {
            detail: err.to_string(),
        })?;

    let protocol = match request.target.protocol {
        WireProtocol::Swd => ProbeWireProtocol::Swd,
        WireProtocol::Jtag => ProbeWireProtocol::Jtag,
    };
    probe
        .select_protocol(protocol)
        .map_err(|err| AppError::ProbeRsFailure {
            detail: err.to_string(),
        })?;

    if let Some(speed_khz) = request.target.speed_khz {
        probe
            .set_speed(speed_khz)
            .map_err(|err| AppError::ProbeRsFailure {
                detail: err.to_string(),
            })?;
    }

    let mut session = if request.target.connect_under_reset {
        probe.attach_under_reset(chip, Permissions::default())
    } else {
        probe.attach(chip, Permissions::default())
    }
    .map_err(|err| AppError::ProbeRsFailure {
        detail: err.to_string(),
    })?;

    let format = probe_rs_format(firmware_format, request.firmware.base_address)?;
    let mut options = DownloadOptions::default();
    options.verify = request.options.verify;
    options.dry_run = request.options.dry_run;
    options.skip_erase = request.options.skip_erase;
    options.do_chip_erase = request.options.allow_erase_all;

    emit_job_event(
        app,
        job_id,
        JobKind::Flash,
        JobStage::Programming,
        Some(0.3),
        "正在烧录固件",
    )?;

    download_file_with_options(
        &mut session,
        Path::new(&request.firmware.path),
        format,
        options,
    )
    .map_err(|err| AppError::ProbeRsFailure {
        detail: err.to_string(),
    })?;

    if request.options.reset_after {
        emit_job_event(
            app,
            job_id,
            JobKind::Flash,
            JobStage::Resetting,
            Some(0.9),
            "正在复位目标芯片",
        )?;

        session
            .core(0)
            .and_then(|mut core| core.reset())
            .map_err(|err| AppError::ProbeRsFailure {
                detail: err.to_string(),
            })?;
    }

    if request.options.halt_after {
        emit_job_event(
            app,
            job_id,
            JobKind::Flash,
            JobStage::Resetting,
            Some(0.9),
            "正在暂停目标芯片",
        )?;

        session
            .core(0)
            .and_then(|mut core| core.halt(Duration::from_millis(500)).map(|_| ()))
            .map_err(|err| AppError::ProbeRsFailure {
                detail: err.to_string(),
            })?;
    }

    Ok(())
}

fn probe_rs_format(
    format: FirmwareFormat,
    base_address: Option<u64>,
) -> Result<probe_rs::flashing::Format> {
    use probe_rs::flashing::{BinOptions, ElfOptions, Format};

    Ok(match format {
        FirmwareFormat::Elf => Format::Elf(ElfOptions::default()),
        FirmwareFormat::Hex => Format::Hex,
        FirmwareFormat::Bin => Format::Bin(BinOptions {
            base_address: Some(base_address.ok_or(AppError::MissingBinBaseAddress)?),
            skip: 0,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use crate::models::{FirmwareInput, FlashOptions, TargetSelection, WireProtocol};

    fn valid_elf_request() -> FlashRequest {
        FlashRequest {
            firmware: FirmwareInput {
                path: "/tmp/app.elf".to_string(),
                format: Some(FirmwareFormat::Elf),
                base_address: None,
            },
            probe: None,
            target: TargetSelection {
                chip: Some("STM32F103C8".to_string()),
                protocol: WireProtocol::Swd,
                speed_khz: None,
                connect_under_reset: false,
            },
            options: FlashOptions {
                verify: true,
                dry_run: false,
                skip_erase: false,
                allow_erase_all: false,
                reset_after: false,
                halt_after: false,
            },
        }
    }

    #[test]
    fn validate_flash_request_should_reject_reset_and_halt_together() {
        let mut request = valid_elf_request();
        request.options.reset_after = true;
        request.options.halt_after = true;

        let err = validate_flash_request(&request).expect_err("request should be rejected");

        assert!(
            matches!(err, AppError::InvalidUserInput { detail } if detail == "reset_after 和 halt_after 不能同时启用")
        );
    }

    #[test]
    fn validate_flash_request_should_accept_valid_elf_request() {
        let format = validate_flash_request(&valid_elf_request()).expect("request should be valid");

        assert_eq!(format, FirmwareFormat::Elf);
    }

    #[test]
    fn validate_flash_request_should_reject_missing_chip() {
        let mut request = valid_elf_request();
        request.target.chip = None;

        let err = validate_flash_request(&request).expect_err("request should be rejected");

        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }

    #[test]
    fn validate_flash_request_should_reject_missing_bin_base_address() {
        let mut request = valid_elf_request();
        request.firmware.path = "/tmp/app.bin".to_string();
        request.firmware.format = Some(FirmwareFormat::Bin);

        let err = validate_flash_request(&request).expect_err("request should be rejected");

        assert!(matches!(err, AppError::MissingBinBaseAddress));
    }

    #[test]
    fn validate_flash_request_should_reject_format_mismatch() {
        let mut request = valid_elf_request();
        request.firmware.path = "/tmp/app.bin".to_string();
        request.firmware.format = Some(FirmwareFormat::Elf);

        let err = validate_flash_request(&request).expect_err("request should be rejected");

        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }

    #[test]
    fn validate_flash_request_should_reject_skip_erase_and_allow_erase_all_together() {
        let mut request = valid_elf_request();
        request.options.skip_erase = true;
        request.options.allow_erase_all = true;

        let err = validate_flash_request(&request).expect_err("request should be rejected");

        assert!(
            matches!(err, AppError::InvalidUserInput { detail } if detail == "skip_erase 和 allow_erase_all 不能同时启用")
        );
    }

    #[test]
    fn validate_flash_request_should_reject_zero_speed() {
        let mut request = valid_elf_request();
        request.target.speed_khz = Some(0);

        let err = validate_flash_request(&request).expect_err("request should be rejected");

        assert!(
            matches!(err, AppError::InvalidUserInput { detail } if detail == "调试速度必须大于 0 kHz")
        );
    }

    #[test]
    fn failed_message_should_include_frontend_safe_invalid_input_detail() {
        let message = failed_message(&AppError::InvalidUserInput {
            detail: "调试速度必须大于 0 kHz".to_string(),
        });

        assert_eq!(message, "用户输入无效：调试速度必须大于 0 kHz");
    }

    #[test]
    fn failed_message_should_not_leak_probe_rs_raw_detail() {
        let message = failed_message(&AppError::ProbeRsFailure {
            detail: "raw probe-rs stack detail".to_string(),
        });

        assert_eq!(
            message,
            "probe-rs 操作失败：probe-rs 返回了错误，完整信息见任务日志"
        );
    }
}
