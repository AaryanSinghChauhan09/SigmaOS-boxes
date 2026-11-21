//! Image build plan — what `arch-boxes` produces.

use crate::disk::{DiskImage, Filesystem};

/// Boot firmware mode for the image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMode {
    /// UEFI only.
    Uefi,
    /// BIOS only (syslinux).
    Bios,
    /// Both UEFI + BIOS.
    Both,
}

/// Output image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// QCOW2 (QEMU).
    Qcow2,
    /// Raw disk image.
    Raw,
    /// Compressed qcow2.
    Qcow2Compressed,
}

/// Package set to install in the image.
#[derive(Debug, Clone, Default)]
pub struct PackageSet {
    /// Base package groups (`base`, `base-devel`).
    pub groups: Vec<String>,
    /// Individual packages.
    pub packages: Vec<String>,
}

/// A complete image build specification.
#[derive(Debug, Clone)]
pub struct ImageBuild {
    /// Release name/tag (e.g. `2025.06.01`).
    pub release: String,
    /// Architecture.
    pub arch: String,
    /// Disk layout.
    pub disk: DiskImage,
    /// Boot mode.
    pub boot_mode: BootMode,
    /// Output format.
    pub format: ImageFormat,
    /// Packages to install.
    pub packages: PackageSet,
    /// Whether to enable SSH by default.
    pub ssh_enabled: bool,
    /// Whether to include the archinstall package.
    pub include_installer: bool,
}

impl ImageBuild {
    /// Convenience: default Arch cloud image build.
    pub fn default_cloud(release: &str, size_gib: u32) -> Self {
        Self {
            release: release.to_string(),
            arch: "x86_64".to_string(),
            disk: DiskImage::default_arch(size_gib, Filesystem::Ext4),
            boot_mode: BootMode::Both,
            format: ImageFormat::Qcow2,
            packages: PackageSet {
                groups: vec!["base".into()],
                packages: vec!["openssh".into(), "qemu-guest-agent".into()],
            },
            ssh_enabled: true,
            include_installer: false,
        }
    }
}

/// Build plan output: commands/artifacts the builder will produce.
#[derive(Debug)]
pub struct BuildPlan {
    /// Output filename.
    pub output_file: String,
    /// pacstrap package list (groups + packages flattened).
    pub pacstrap_pkgs: Vec<String>,
    /// Whether UEFI systemd-boot entries needed.
    pub needs_uefi_boot: bool,
    /// Whether BIOS syslinux needed.
    pub needs_bios_boot: bool,
    /// mkosi-style mkosi.conf summary lines.
    pub mkosi_conf: Vec<String>,
}

/// Generate the build plan for an [`ImageBuild`].
pub fn build_plan(build: &ImageBuild) -> Result<BuildPlan, crate::BoxesError> {
    build.disk.validate()?;
    let mut pacstrap = build.packages.groups.clone();
    pacstrap.extend(build.packages.packages.clone());
    if build.ssh_enabled && !pacstrap.iter().any(|p| p == "openssh") {
        pacstrap.push("openssh".into());
    }
    if build.include_installer {
        pacstrap.push("archinstall".into());
    }
    let ext = match build.format {
        ImageFormat::Qcow2 => "qcow2",
        ImageFormat::Raw => "img",
        ImageFormat::Qcow2Compressed => "qcow2.xz",
    };
    let mkosi_conf = vec![
        format!("[Output] Format={}", match build.format {
            ImageFormat::Qcow2 => "qcow2",
            ImageFormat::Raw => "raw",
            ImageFormat::Qcow2Compressed => "compressed",
        }),
        format!("[Content] Packages={}", pacstrap.join(" ")),
        format!("[Host] Architecture={}", build.arch),
    ];
    Ok(BuildPlan {
        output_file: format!("archlinux-{}-{}-{}.{}", build.release, build.arch, build.boot_mode_name(), ext),
        pacstrap_pkgs: pacstrap,
        needs_uefi_boot: matches!(build.boot_mode, BootMode::Uefi | BootMode::Both),
        needs_bios_boot: matches!(build.boot_mode, BootMode::Bios | BootMode::Both),
        mkosi_conf,
    })
}

impl ImageBuild {
    fn boot_mode_name(&self) -> &'static str {
        match self.boot_mode {
            BootMode::Uefi => "uefi",
            BootMode::Bios => "bios",
            BootMode::Both => "dual",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cloud_plan() {
        let b = ImageBuild::default_cloud("2025.06.01", 10);
        let plan = build_plan(&b).unwrap();
        assert!(plan.needs_uefi_boot && plan.needs_bios_boot);
        assert!(plan.pacstrap_pkgs.contains(&"openssh".to_string()));
        assert!(plan.output_file.contains("qcow2"));
    }

    #[test]
    fn test_disk_validate_overlap() {
        let mut d = DiskImage::default_arch(1, Filesystem::Ext4);
        d.partitions[1].start_mib = 100; // overlaps esp (1+512=513)
        assert!(d.validate().is_err());
    }
}
