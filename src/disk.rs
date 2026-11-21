//! Disk image model — GPT partition layout for Arch images.

/// Filesystem type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filesystem {
    /// ext4.
    Ext4,
    /// btrfs.
    Btrfs,
    /// xfs.
    Xfs,
    /// vfat (EFI system partition).
    Vfat,
}

impl Filesystem {
    /// mkfs command name.
    pub fn mkfs_cmd(&self) -> &'static str {
        match self {
            Self::Ext4 => "mkfs.ext4",
            Self::Btrfs => "mkfs.btrfs",
            Self::Xfs => "mkfs.xfs",
            Self::Vfat => "mkfs.vfat",
        }
    }
}

/// A GPT partition.
#[derive(Debug, Clone)]
pub struct Partition {
    /// Partition number (1-based).
    pub number: u32,
    /// Partition name (GPT name).
    pub name: String,
    /// Start offset in MiB.
    pub start_mib: u64,
    /// Size in MiB (0 = rest of disk).
    pub size_mib: u64,
    /// Filesystem.
    pub fs: Filesystem,
    /// Mount point inside the image.
    pub mount_point: String,
    /// Whether this is the ESP/boot partition.
    pub boot: bool,
}

/// Partition table type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionTable {
    /// GPT.
    Gpt,
    /// MBR/DOS.
    Mbr,
}

/// A disk image layout.
#[derive(Debug, Clone)]
pub struct DiskImage {
    /// Total size in GiB.
    pub size_gib: u32,
    /// Partition table.
    pub table: PartitionTable,
    /// Partitions in order.
    pub partitions: Vec<Partition>,
}

impl DiskImage {
    /// Default Arch cloud image layout:
    /// p1 = ESP (vfat, /boot, 512MiB), p2 = root (ext4 or btrfs, rest).
    pub fn default_arch(size_gib: u32, fs: Filesystem) -> Self {
        Self {
            size_gib,
            table: PartitionTable::Gpt,
            partitions: vec![
                Partition {
                    number: 1, name: "esp".into(), start_mib: 1,
                    size_mib: 512, fs: Filesystem::Vfat,
                    mount_point: "/boot".into(), boot: true,
                },
                Partition {
                    number: 2, name: "root".into(), start_mib: 513,
                    size_mib: 0, fs, mount_point: "/".into(), boot: false,
                },
            ],
        }
    }

    /// Validate partition layout (no overlaps, sizes fit).
    pub fn validate(&self) -> Result<(), crate::BoxesError> {
        let disk_mib = self.size_gib as u64 * 1024;
        let mut cursor = 0u64;
        for p in &self.partitions {
            if p.start_mib < cursor {
                return Err(crate::BoxesError::InvalidConfig(format!(
                    "partition {} overlaps previous",
                    p.number
                )));
            }
            if p.size_mib > 0 && p.start_mib + p.size_mib > disk_mib {
                return Err(crate::BoxesError::InvalidConfig(format!(
                    "partition {} exceeds disk size",
                    p.number
                )));
            }
            cursor = p.start_mib + p.size_mib.max(1);
        }
        Ok(())
    }
}
