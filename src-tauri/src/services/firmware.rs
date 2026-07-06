use crate::{
    error::{AppError, Result},
    models::{FirmwareFormat, FirmwareInput},
};
use std::path::Path;

/// 根据文件扩展名推断固件格式。
pub fn detect_format(path: &Path) -> Result<FirmwareFormat> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| AppError::UnsupportedFirmwareFormat {
            path: path.display().to_string(),
        })?;

    match extension.as_str() {
        "elf" | "axf" => Ok(FirmwareFormat::Elf),
        "hex" => Ok(FirmwareFormat::Hex),
        "bin" => Ok(FirmwareFormat::Bin),
        _ => Err(AppError::UnsupportedFirmwareFormat {
            path: path.display().to_string(),
        }),
    }
}

/// 校验固件输入并返回最终采用的固件格式。
pub fn validate_firmware(input: &FirmwareInput) -> Result<FirmwareFormat> {
    if input.path.trim().is_empty() {
        return Err(AppError::InvalidUserInput {
            detail: "固件路径不能为空".to_string(),
        });
    }

    let detected = detect_format(Path::new(&input.path))?;

    if let Some(explicit) = input.format
        && explicit != detected
    {
        return Err(AppError::InvalidUserInput {
            detail: format!("固件格式与文件扩展名不匹配：选择为 {explicit:?}，检测为 {detected:?}"),
        });
    }

    if detected == FirmwareFormat::Bin {
        let address = input.base_address.ok_or(AppError::MissingBinBaseAddress)?;
        validate_flash_address(address)?;
    }

    Ok(detected)
}

/// 解析用户输入的十进制或 `0x` 前缀十六进制地址。
pub fn parse_address(value: &str) -> Result<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidFirmwareAddress {
            detail: "地址不能为空".to_string(),
        });
    }

    let parsed = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
    } else {
        trimmed.parse::<u64>()
    };

    parsed.map_err(|err| AppError::InvalidFirmwareAddress {
        detail: format!("无法解析地址 `{trimmed}`: {err}"),
    })
}

fn validate_flash_address(address: u64) -> Result<()> {
    if !address.is_multiple_of(4) {
        return Err(AppError::InvalidFirmwareAddress {
            detail: "Flash 地址必须 4 字节对齐".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_format_should_accept_elf_extension() {
        let format = detect_format(Path::new("/tmp/app.elf")).unwrap();
        assert_eq!(format, FirmwareFormat::Elf);
    }

    #[test]
    fn detect_format_should_accept_hex_extension() {
        let format = detect_format(Path::new("/tmp/app.hex")).unwrap();
        assert_eq!(format, FirmwareFormat::Hex);
    }

    #[test]
    fn detect_format_should_accept_bin_extension() {
        let format = detect_format(Path::new("/tmp/app.bin")).unwrap();
        assert_eq!(format, FirmwareFormat::Bin);
    }

    #[test]
    fn detect_format_should_accept_uppercase_extension() {
        let format = detect_format(Path::new("/tmp/APP.HEX")).unwrap();
        assert_eq!(format, FirmwareFormat::Hex);
    }

    #[test]
    fn detect_format_should_reject_unsupported_extension() {
        let err = detect_format(Path::new("/tmp/app.txt")).unwrap_err();
        assert!(matches!(err, AppError::UnsupportedFirmwareFormat { .. }));
    }

    #[test]
    fn validate_firmware_should_reject_bin_without_base_address() {
        let input = FirmwareInput {
            path: "/tmp/app.bin".to_string(),
            format: None,
            base_address: None,
        };
        let err = validate_firmware(&input).unwrap_err();
        assert!(matches!(err, AppError::MissingBinBaseAddress));
    }

    #[test]
    fn validate_firmware_should_reject_explicit_format_mismatch() {
        let input = FirmwareInput {
            path: "/tmp/app.bin".to_string(),
            format: Some(FirmwareFormat::Elf),
            base_address: None,
        };
        let err = validate_firmware(&input).unwrap_err();
        assert!(matches!(err, AppError::InvalidUserInput { .. }));
    }

    #[test]
    fn validate_firmware_should_accept_bin_with_zero_base_address() {
        let input = FirmwareInput {
            path: "/tmp/app.bin".to_string(),
            format: None,
            base_address: Some(0),
        };
        let format = validate_firmware(&input).unwrap();
        assert_eq!(format, FirmwareFormat::Bin);
    }

    #[test]
    fn validate_firmware_should_reject_unaligned_bin_base_address() {
        let input = FirmwareInput {
            path: "/tmp/app.bin".to_string(),
            format: None,
            base_address: Some(3),
        };
        let err = validate_firmware(&input).unwrap_err();
        assert!(matches!(err, AppError::InvalidFirmwareAddress { .. }));
    }

    #[test]
    fn validate_firmware_should_ignore_non_bin_base_address() {
        let input = FirmwareInput {
            path: "/tmp/app.elf".to_string(),
            format: None,
            base_address: Some(3),
        };
        let format = validate_firmware(&input).unwrap();
        assert_eq!(format, FirmwareFormat::Elf);
    }

    #[test]
    fn parse_address_should_accept_hex_prefix() {
        let address = parse_address("0x08000000").unwrap();
        assert_eq!(address, 0x0800_0000);
    }

    #[test]
    fn parse_address_should_reject_empty_string() {
        let err = parse_address("").unwrap_err();
        assert!(matches!(err, AppError::InvalidFirmwareAddress { .. }));
    }
}
