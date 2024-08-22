use anyhow::Result;
use log::{debug, warn};
use nix::mount::{mount, MsFlags};
use std::ffi::CString;
use std::path::PathBuf;

pub fn tmpfs(target: PathBuf) -> Result<()> {
    mount(
        Some("tmpfs"),
        &target,
        Some("tmpfs"),
        MsFlags::empty(),
        // We use the default size which is `physical ram without swap / 2`
        // TODO: Ideally we would use the `noswap` option if available on the current kernel so performance
        // will be more predictable and print a warning otherwise.
        Some("mode=1777"),
    )?;

    Ok(())
}

pub fn squashfs(loop_device_path: PathBuf, target: PathBuf) -> Result<()> {
    mount::<PathBuf, PathBuf, str, str>(
        Some(&loop_device_path),
        &target,
        Some("squashfs"),
        MsFlags::MS_RDONLY,
        None,
    )?;

    Ok(())
}

pub fn overlayfs(target: PathBuf) -> Result<()> {
    let options = CString::new(format!(
        "lowerdir={},upperdir={},workdir={},xino=off",
        target.join("lower").display(),
        target.join("upper").display(),
        target.join("work").display()
    ))?;

    mount(
        Some("overlay"),
        &target.join("merged"),
        Some("overlay"),
        MsFlags::empty(),
        Some(options.as_c_str()),
    )?;

    Ok(())
}

pub fn essential_system_filesystems(target: PathBuf) -> Result<()> {
    mount::<str, PathBuf, str, str>(
        Some("proc"),
        &target.join("proc"),
        Some("proc"),
        MsFlags::empty(),
        None,
    )?;

    mount::<str, PathBuf, str, str>(
        Some("sysfs"),
        &target.join("sys"),
        Some("sysfs"),
        MsFlags::empty(),
        None,
    )?;

    mount::<str, PathBuf, str, str>(
        Some("/dev"),
        &target.join("dev"),
        None,
        MsFlags::MS_BIND,
        None,
    )?;

    mount::<str, PathBuf, str, str>(
        Some("devpts"),
        &target.join("dev/pts"),
        Some("devpts"),
        MsFlags::empty(),
        None,
    )?;

    Ok(())
}

pub fn non_essential_system_filesystems(target: PathBuf) -> Result<()> {
    let fstypes_and_mountpoints: Vec<(&str, PathBuf)> = vec![
        ("mqueue", target.join("dev/mqueue")),
        // For some reason docker mounts it under the source name "cgroup",
        // but ubuntu calls it "cgroup2" so we go that way.
        ("cgroup2", target.join("sys/fs/cgroup")),
        ("bpf", target.join("sys/fs/bpf")),
        ("configfs", target.join("sys/kernel/config")),
        ("tracefs", target.join("sys/kernel/tracing")),
        ("efivarfs", target.join("sys/firmware/efi/efivars")),
        ("securityfs", target.join("sys/kernel/security")),
        ("pstore", target.join("sys/fs/pstore")),
        ("hugetlbfs", target.join("dev/hugepages")),
        ("binfmt_misc", target.join("proc/sys/fs/binfmt_misc")),
        ("fusectl", target.join("sys/fs/fuse/connections")),
        ("debugfs", target.join("sys/kernel/debug")),
        // Apparantly glibc expects /dev/shm to be a tmpfs but we are based on Alpine
        // so no need for that.
    ];

    for (fstype, mountpoint) in fstypes_and_mountpoints.iter() {
        match mount::<str, PathBuf, str, str>(
            Some(fstype),
            &mountpoint,
            Some(fstype),
            MsFlags::empty(),
            None,
        ) {
            Ok(()) => {
                debug!("mounted {} filesystem on {}", fstype, mountpoint.display());
            }
            Err(err) => {
                warn!(
                    "failed mounting {} filesystem on {}: {:?}",
                    fstype,
                    mountpoint.display(),
                    err
                );
            }
        }
    }

    Ok(())
}

pub fn network_configuration(target: PathBuf) -> Result<()> {
    let network_configuration_files: [&str; 3] = ["etc/resolv.conf", "etc/hostname", "etc/hosts"];

    for file in network_configuration_files {
        mount::<PathBuf, PathBuf, str, str>(
            Some(&PathBuf::from("/").join(file)),
            &target.join(file),
            None,
            MsFlags::MS_BIND,
            None,
        )?;

        mount::<str, PathBuf, str, str>(
            None,
            &target.join(file),
            None,
            MsFlags::MS_REMOUNT | MsFlags::MS_BIND | MsFlags::MS_RDONLY,
            None,
        )?;
    }

    Ok(())
}
