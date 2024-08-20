#![feature(const_intrinsic_copy)]
#![feature(const_mut_refs)]

mod mount;
use anyhow::Result;
use goblin::elf::Elf;
use log::info;
use loopdev::{LoopControl, LoopDevice};
use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::sys::stat;
use nix::unistd;
use nix::unistd::execve;
use nix::unistd::pivot_root;
use std::ffi::{CStr, CString};
use std::io::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;

const SQUASHFS_BYTES: &[u8] = include_bytes!("../../container/alpine-snow.squashfs");

#[used]
#[link_section = ".squashfs"]
#[allow(long_running_const_eval)]
pub static SQUASHFS_SECTION: [u8; SQUASHFS_BYTES.len()] = {
    let mut bytes = [0u8; SQUASHFS_BYTES.len()];
    unsafe {
        std::ptr::copy_nonoverlapping(
            SQUASHFS_BYTES.as_ptr(),
            &mut bytes as *mut _,
            SQUASHFS_BYTES.len(),
        )
    };
    bytes
};

fn get_squashfs_section_address() -> Result<Option<u64>> {
    let mut file = std::fs::File::open("/proc/self/exe")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let elf = Elf::parse(&buffer)?;

    for section in elf.section_headers.iter() {
        let name = elf
            .shdr_strtab
            .get_at(section.sh_name)
            .expect("failed to get section name");
        if name == ".squashfs" {
            return Ok(Some(section.sh_addr));
        }
    }

    Ok(None)
}

fn enter_new_mount_ns() -> Result<()> {
    unshare(CloneFlags::CLONE_NEWNS)?;
    mount::<str, str, str, str>(None, "/", None, MsFlags::MS_PRIVATE | MsFlags::MS_REC, None)?;

    Ok(())
}

fn create_loop_device(target_file: PathBuf, offset: u64) -> Result<LoopDevice> {
    let loop_control = LoopControl::open()?;
    let loop_device = loop_control.next_free()?;

    loop_device
        .with()
        .offset(offset)
        .read_only(true)
        .attach(target_file)?;

    Ok(loop_device)
}

fn create_overlayfs_directories(target: PathBuf) -> Result<()> {
    unistd::mkdir(&target.join("lower"), stat::Mode::S_IRWXU)?;
    unistd::mkdir(&target.join("work"), stat::Mode::S_IRWXU)?;
    unistd::mkdir(&target.join("upper"), stat::Mode::S_IRWXU)?;
    unistd::mkdir(&target.join("merged"), stat::Mode::S_IRWXU)?;

    Ok(())
}

fn change_root_filesystem(new_root: PathBuf) -> Result<()> {
    let put_old = new_root.join("mnt/root");
    unistd::mkdir(&put_old, stat::Mode::S_IRWXU)?;

    pivot_root(&new_root, &put_old)?;

    unistd::chdir("/")?;

    Ok(())
}

fn exec_zsh() -> Result<()> {
    let sbin_init = CString::new("/bin/zsh")?;
    let mut args_cstring: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).map_or(CString::default(), |res| res))
        .collect::<Vec<CString>>();

    let _ = std::mem::replace(&mut args_cstring[0], CString::new("zsh")?);

    let mut args_cstr = Vec::<&CStr>::new();

    for arg_cstring in args_cstring.iter() {
        args_cstr.push(arg_cstring.as_c_str())
    }

    execve::<&CStr, &CStr>(sbin_init.as_c_str(), &args_cstr, &[])?;

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();

    info!(
        "squashfs taste: {}, pid: {}",
        // hex::encode(Sha256::digest(SQUASHFS_SECTION)),
        SQUASHFS_SECTION[std::process::id() as usize],
        std::process::id()
    );

    // This directory will not be available for us anymore :(
    // but its pretty much useless for our process.
    //
    // Another nice benefit it has is that all mounts we create another it
    // will be automaticaly lazily unmounted when this process is reaped.
    let useless_dir = PathBuf::from_str("/proc/self/fd")?;

    let squashfs_offset = get_squashfs_section_address()?.expect("squashfs section not found");

    info!("entering new mount ns");
    enter_new_mount_ns()?;

    info!("creating loop device on self exe, squashfs offset {squashfs_offset}");
    let loop_device = create_loop_device("/proc/self/exe".into(), squashfs_offset)?;

    let loop_device_path = loop_device
        .path()
        .expect("failed to get path of loop device!");
    info!("using loop device {}", loop_device_path.display());

    info!("mounting tmpfs on {:?}", useless_dir.clone());
    mount::tmpfs(useless_dir.clone())?;

    info!(
        "creating overlayfs directories on {:?}",
        useless_dir.clone()
    );
    create_overlayfs_directories(useless_dir.clone())?;

    info!(
        "mounting squashfs ro on {}",
        useless_dir.join("lower").display()
    );
    mount::squashfs_ro(loop_device_path, useless_dir.join("lower"))?;

    info!("mounting overlayfs using {}", useless_dir.display());
    mount::overlayfs(useless_dir.clone())?;

    info!(
        "mounting /proc /sys /dev /dev/pts on {}",
        useless_dir.join("merged").display()
    );
    mount::essential_system_filesystems(useless_dir.join("merged"))?;

    info!(
        "mounting non essential system filesystems on {}",
        useless_dir.join("merged").display()
    );
    mount::non_essential_system_filesystems(useless_dir.join("merged"))?;

    info!(
        "changing root filesystem to {}",
        useless_dir.join("merged").display()
    );
    change_root_filesystem(useless_dir.join("merged"))?;

    info!("exec-ing zsh bye!");
    let _ = exec_zsh()?;

    Ok(())
}
