//! VRT configuration types.

use serde::{Deserialize, Serialize};
use vize_carton::{String, ToCompactString};

use super::preset::ViewportPreset;
use crate::types::ViewportConfig;

/// VRT configuration parsed from musea.config.ts or vite.config.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrtConfig {
    /// Whether VRT is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Snapshot storage directory
    #[serde(default = "default_snapshot_dir")]
    pub snapshot_dir: String,

    /// Pixel difference threshold
    #[serde(default)]
    pub threshold: VrtThreshold,

    /// Viewports to capture
    #[serde(default = "default_viewports")]
    pub viewports: Vec<ViewportConfig>,

    /// Browser options
    #[serde(default)]
    pub browser: BrowserConfig,

    /// Capture options
    #[serde(default)]
    pub capture: CaptureConfig,

    /// Comparison options
    #[serde(default)]
    pub comparison: ComparisonConfig,

    /// CI-specific options
    #[serde(default)]
    pub ci: CiConfig,
}

/// VRT options (simpler version for programmatic use).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrtOptions {
    /// Snapshot storage directory
    pub snapshot_dir: Option<String>,

    /// Pixel difference threshold percentage (0-100)
    pub threshold: Option<f64>,

    /// Viewports to capture
    pub viewports: Option<Vec<ViewportConfig>>,
}

/// Threshold configuration for pixel comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrtThreshold {
    /// Maximum allowed difference percentage (0-100)
    #[serde(default = "default_percentage")]
    pub percentage: f64,

    /// Maximum allowed different pixels (absolute)
    #[serde(default)]
    pub pixels: Option<u32>,

    /// Color sensitivity (0-1, lower = more strict)
    #[serde(default = "default_color_sensitivity")]
    pub color_sensitivity: f64,
}

/// Browser configuration for VRT.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserConfig {
    /// Browser to use
    #[serde(default = "default_browser")]
    pub name: String,

    /// Run in headless mode
    #[serde(default = "default_true")]
    pub headless: bool,

    /// Slow motion delay (ms) for debugging
    #[serde(default)]
    pub slow_mo: Option<u32>,

    /// Browser timeout (ms)
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

/// Screenshot capture configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureConfig {
    /// Capture full page vs viewport only
    #[serde(default)]
    pub full_page: bool,

    /// Wait for network idle before capture
    #[serde(default = "default_true")]
    pub wait_for_network: bool,

    /// Additional wait time after load (ms)
    #[serde(default = "default_settle_time")]
    pub settle_time: u32,

    /// CSS selector to wait for
    #[serde(default = "default_wait_selector")]
    pub wait_selector: String,

    /// Elements to hide before capture (CSS selectors)
    #[serde(default)]
    pub hide_elements: Vec<String>,

    /// Elements to mask before capture (CSS selectors)
    #[serde(default)]
    pub mask_elements: Vec<String>,
}

/// Image comparison configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonConfig {
    /// Anti-aliasing detection
    #[serde(default = "default_true")]
    pub anti_aliasing: bool,

    /// Alpha channel comparison
    #[serde(default = "default_true")]
    pub alpha: bool,

    /// Diff image output format
    #[serde(default = "default_diff_style")]
    pub diff_style: DiffStyle,

    /// Diff highlight color
    #[serde(default)]
    pub diff_color: Option<RgbColor>,
}

/// CI-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CiConfig {
    /// Fail build on any diff
    #[serde(default = "default_true")]
    pub fail_on_diff: bool,

    /// Auto-update baselines on main branch
    #[serde(default)]
    pub auto_update_on_main: bool,

    /// Generate JSON report for CI
    #[serde(default = "default_true")]
    pub json_report: bool,

    /// Retry failed tests
    #[serde(default)]
    pub retries: u32,
}

/// Diff image style.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffStyle {
    /// Overlay diff on grayscale background
    #[default]
    Overlay,
    /// Side-by-side comparison
    SideBySide,
    /// Diff only (no background)
    DiffOnly,
    /// Animated GIF comparison
    Animated,
}

/// RGB color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// Default value functions

fn default_enabled() -> bool {
    true
}

fn default_snapshot_dir() -> String {
    ".musea/snapshots".to_compact_string()
}

fn default_viewports() -> Vec<ViewportConfig> {
    vec![
        ViewportPreset::Desktop.into(),
        ViewportPreset::Mobile.into(),
    ]
}

fn default_percentage() -> f64 {
    0.1
}

fn default_color_sensitivity() -> f64 {
    0.1
}

fn default_browser() -> String {
    "chromium".to_compact_string()
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u32 {
    30000
}

fn default_settle_time() -> u32 {
    100
}

fn default_wait_selector() -> String {
    ".musea-variant".to_compact_string()
}

fn default_diff_style() -> DiffStyle {
    DiffStyle::Overlay
}

// Default implementations

impl Default for VrtConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            snapshot_dir: default_snapshot_dir(),
            threshold: VrtThreshold::default(),
            viewports: default_viewports(),
            browser: BrowserConfig::default(),
            capture: CaptureConfig::default(),
            comparison: ComparisonConfig::default(),
            ci: CiConfig::default(),
        }
    }
}

impl Default for VrtThreshold {
    fn default() -> Self {
        Self {
            percentage: default_percentage(),
            pixels: None,
            color_sensitivity: default_color_sensitivity(),
        }
    }
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            name: default_browser(),
            headless: default_true(),
            slow_mo: None,
            timeout: default_timeout(),
        }
    }
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            full_page: false,
            wait_for_network: default_true(),
            settle_time: default_settle_time(),
            wait_selector: default_wait_selector(),
            hide_elements: Vec::new(),
            mask_elements: Vec::new(),
        }
    }
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            anti_aliasing: default_true(),
            alpha: default_true(),
            diff_style: default_diff_style(),
            diff_color: None,
        }
    }
}

impl Default for CiConfig {
    fn default() -> Self {
        Self {
            fail_on_diff: default_true(),
            auto_update_on_main: false,
            json_report: default_true(),
            retries: 0,
        }
    }
}

impl Default for RgbColor {
    fn default() -> Self {
        // Default diff color: red
        Self { r: 255, g: 0, b: 0 }
    }
}

impl VrtConfig {
    /// Create config from VrtOptions (partial configuration).
    pub fn from_options(options: VrtOptions) -> Self {
        let mut config = Self::default();

        if let Some(dir) = options.snapshot_dir {
            config.snapshot_dir = dir;
        }
        if let Some(threshold) = options.threshold {
            config.threshold.percentage = threshold;
        }
        if let Some(viewports) = options.viewports {
            config.viewports = viewports;
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::{BrowserConfig, DiffStyle, VrtConfig, VrtOptions, VrtThreshold};

    #[test]
    fn test_vrt_config_default() {
        let config = VrtConfig::default();
        assert!(config.enabled);
        assert_eq!(config.snapshot_dir, ".musea/snapshots");
        assert_eq!(config.threshold.percentage, 0.1);
        assert_eq!(config.viewports.len(), 2);
    }

    #[test]
    fn test_vrt_config_from_options() {
        let options = VrtOptions {
            snapshot_dir: Some("custom/snapshots".into()),
            threshold: Some(0.5),
            viewports: None,
        };

        let config = VrtConfig::from_options(options);
        assert_eq!(config.snapshot_dir, "custom/snapshots");
        assert_eq!(config.threshold.percentage, 0.5);
        assert_eq!(config.viewports.len(), 2); // default viewports
    }

    #[test]
    fn test_vrt_threshold_default() {
        let threshold = VrtThreshold::default();
        assert_eq!(threshold.percentage, 0.1);
        assert_eq!(threshold.color_sensitivity, 0.1);
    }

    #[test]
    fn test_browser_config_default() {
        let browser = BrowserConfig::default();
        assert_eq!(browser.name, "chromium");
        assert!(browser.headless);
        assert_eq!(browser.timeout, 30000);
    }

    #[test]
    fn test_diff_style_default() {
        assert!(matches!(DiffStyle::default(), DiffStyle::Overlay));
    }
}
