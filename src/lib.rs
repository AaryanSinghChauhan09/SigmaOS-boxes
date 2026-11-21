//! SigmaOS boxes component.
//!
//! Rust implementation of `archlinux/boxes` (arch-boxes on GitLab): the
//! tool that builds QCOW2 disk images of Arch Linux for VMs/cloud.
//!
//! Upstream `arch-boxes` is a bash tool using mkosi/systemd-repart to
//! produce bootable images. This port implements the image-build model:
//! disk layout (partitions, filesystems), package selection, boot mode,
//! and the mkosi-style manifest the builder consumes.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod disk;
pub mod error;
pub mod image;

pub use disk::{DiskImage, Filesystem, Partition, PartitionTable};
pub use error::BoxesError;
pub use image::{BootMode, ImageBuild, ImageFormat, PackageSet, build_plan};
