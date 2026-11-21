//! Integration tests for sigmaos-boxes.

use sigmaos_boxes::{BootMode, DiskImage, Filesystem, ImageBuild, ImageFormat, build_plan};

#[test]
fn test_cloud_image_plan() {
    let b = ImageBuild::default_cloud("2025.06.01", 20);
    let plan = build_plan(&b).unwrap();
    assert!(plan.pacstrap_pkgs.contains(&"base".to_string()));
    assert!(plan.needs_uefi_boot);
    assert!(plan.needs_bios_boot);
    assert!(plan.mkosi_conf.iter().any(|l| l.contains("Packages=")));
}

#[test]
fn test_bios_only() {
    let mut b = ImageBuild::default_cloud("r1", 10);
    b.boot_mode = BootMode::Bios;
    let plan = build_plan(&b).unwrap();
    assert!(!plan.needs_uefi_boot);
    assert!(plan.needs_bios_boot);
}

#[test]
fn test_disk_layout_default() {
    let d = DiskImage::default_arch(10, Filesystem::Btrfs);
    assert_eq!(d.partitions.len(), 2);
    assert!(d.partitions[0].boot);
    assert_eq!(d.partitions[1].fs, Filesystem::Btrfs);
    assert!(d.validate().is_ok());
}

#[test]
fn test_installer_package_included() {
    let mut b = ImageBuild::default_cloud("r2", 10);
    b.include_installer = true;
    let plan = build_plan(&b).unwrap();
    assert!(plan.pacstrap_pkgs.contains(&"archinstall".to_string()));
}
