//! Viewport presets for VRT.
//!
//! Common device viewport configurations.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::types::ViewportConfig;

/// Predefined viewport presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewportPreset {
    // Desktop
    Desktop,
    DesktopHd,
    Desktop4k,

    // Tablet
    TabletPortrait,
    TabletLandscape,
    IpadPro,

    // Mobile
    Mobile,
    MobileLandscape,
    IphoneSe,
    Iphone14,
    Iphone14Pro,
    Iphone14ProMax,
    Pixel7,

    // Custom
    Custom,
}

impl ViewportPreset {
    /// Get the viewport configuration for this preset.
    #[inline]
    pub const fn config(&self) -> ViewportConfig {
        match self {
            // Desktop
            Self::Desktop => ViewportConfig {
                width: 1280,
                height: 720,
                device_scale_factor: Some(1.0),
            },
            Self::DesktopHd => ViewportConfig {
                width: 1920,
                height: 1080,
                device_scale_factor: Some(1.0),
            },
            Self::Desktop4k => ViewportConfig {
                width: 3840,
                height: 2160,
                device_scale_factor: Some(1.0),
            },

            // Tablet
            Self::TabletPortrait => ViewportConfig {
                width: 768,
                height: 1024,
                device_scale_factor: Some(2.0),
            },
            Self::TabletLandscape => ViewportConfig {
                width: 1024,
                height: 768,
                device_scale_factor: Some(2.0),
            },
            Self::IpadPro => ViewportConfig {
                width: 1024,
                height: 1366,
                device_scale_factor: Some(2.0),
            },

            // Mobile
            Self::Mobile => ViewportConfig {
                width: 375,
                height: 667,
                device_scale_factor: Some(2.0),
            },
            Self::MobileLandscape => ViewportConfig {
                width: 667,
                height: 375,
                device_scale_factor: Some(2.0),
            },
            Self::IphoneSe => ViewportConfig {
                width: 375,
                height: 667,
                device_scale_factor: Some(2.0),
            },
            Self::Iphone14 => ViewportConfig {
                width: 390,
                height: 844,
                device_scale_factor: Some(3.0),
            },
            Self::Iphone14Pro => ViewportConfig {
                width: 393,
                height: 852,
                device_scale_factor: Some(3.0),
            },
            Self::Iphone14ProMax => ViewportConfig {
                width: 430,
                height: 932,
                device_scale_factor: Some(3.0),
            },
            Self::Pixel7 => ViewportConfig {
                width: 412,
                height: 915,
                device_scale_factor: Some(2.625),
            },

            // Custom - returns desktop as fallback
            Self::Custom => ViewportConfig {
                width: 1280,
                height: 720,
                device_scale_factor: Some(1.0),
            },
        }
    }

    /// Get the name of this preset.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::DesktopHd => "desktop-hd",
            Self::Desktop4k => "desktop-4k",
            Self::TabletPortrait => "tablet-portrait",
            Self::TabletLandscape => "tablet-landscape",
            Self::IpadPro => "ipad-pro",
            Self::Mobile => "mobile",
            Self::MobileLandscape => "mobile-landscape",
            Self::IphoneSe => "iphone-se",
            Self::Iphone14 => "iphone-14",
            Self::Iphone14Pro => "iphone-14-pro",
            Self::Iphone14ProMax => "iphone-14-pro-max",
            Self::Pixel7 => "pixel-7",
            Self::Custom => "custom",
        }
    }
}

impl FromStr for ViewportPreset {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "desktop" => Ok(Self::Desktop),
            "desktop-hd" | "desktophd" => Ok(Self::DesktopHd),
            "desktop-4k" | "desktop4k" => Ok(Self::Desktop4k),
            "tablet" | "tablet-portrait" => Ok(Self::TabletPortrait),
            "tablet-landscape" => Ok(Self::TabletLandscape),
            "ipad-pro" | "ipadpro" => Ok(Self::IpadPro),
            "mobile" => Ok(Self::Mobile),
            "mobile-landscape" => Ok(Self::MobileLandscape),
            "iphone-se" | "iphonese" => Ok(Self::IphoneSe),
            "iphone-14" | "iphone14" => Ok(Self::Iphone14),
            "iphone-14-pro" | "iphone14pro" => Ok(Self::Iphone14Pro),
            "iphone-14-pro-max" | "iphone14promax" => Ok(Self::Iphone14ProMax),
            "pixel-7" | "pixel7" => Ok(Self::Pixel7),
            _ => Err(()),
        }
    }
}

impl From<ViewportPreset> for ViewportConfig {
    #[inline]
    fn from(preset: ViewportPreset) -> Self {
        preset.config()
    }
}

/// All available viewport presets.
pub const PRESET_VIEWPORTS: &[ViewportPreset] = &[
    ViewportPreset::Desktop,
    ViewportPreset::DesktopHd,
    ViewportPreset::Desktop4k,
    ViewportPreset::TabletPortrait,
    ViewportPreset::TabletLandscape,
    ViewportPreset::IpadPro,
    ViewportPreset::Mobile,
    ViewportPreset::MobileLandscape,
    ViewportPreset::IphoneSe,
    ViewportPreset::Iphone14,
    ViewportPreset::Iphone14Pro,
    ViewportPreset::Iphone14ProMax,
    ViewportPreset::Pixel7,
];

#[cfg(test)]
mod tests {
    use super::{ViewportPreset, PRESET_VIEWPORTS};
    use crate::types::ViewportConfig;
    use std::str::FromStr;

    #[test]
    fn test_desktop_preset() {
        let config = ViewportPreset::Desktop.config();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
    }

    #[test]
    fn test_mobile_preset() {
        let config = ViewportPreset::Mobile.config();
        assert_eq!(config.width, 375);
        assert_eq!(config.height, 667);
    }

    #[test]
    fn test_preset_from_str() {
        assert_eq!(
            ViewportPreset::from_str("desktop"),
            Ok(ViewportPreset::Desktop)
        );
        assert_eq!(
            ViewportPreset::from_str("mobile"),
            Ok(ViewportPreset::Mobile)
        );
        assert_eq!(
            ViewportPreset::from_str("iphone-14-pro"),
            Ok(ViewportPreset::Iphone14Pro)
        );
        assert_eq!(ViewportPreset::from_str("unknown"), Err(()));
    }

    #[test]
    fn test_preset_into_config() {
        let config: ViewportConfig = ViewportPreset::Iphone14.into();
        assert_eq!(config.width, 390);
        assert_eq!(config.height, 844);
        assert_eq!(config.device_scale_factor, Some(3.0));
    }

    #[test]
    fn test_preset_viewports_count() {
        assert_eq!(PRESET_VIEWPORTS.len(), 13);
    }
}
