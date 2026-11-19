//! Give memory the system allocator keeps after frees back to the system
//! ([`luz_engine::memory::set_release_hook`]). The platform call lives in
//! `luz-sysmem`, the workspace's only crate allowed `unsafe`.

/// Install the hook.
pub fn install() {
    luz_engine::memory::set_release_hook(release);
}

fn release() {
    let _ = luz_sysmem::release_free_memory();
}
