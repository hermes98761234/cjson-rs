pub mod merge_patch;
pub mod patch;
pub mod pointer;
pub mod sort;

pub use merge_patch::{
    generate_merge_patch, generate_merge_patch_case_sensitive, merge_patch,
    merge_patch_case_sensitive,
};
pub use patch::{
    add_patch_to_array, apply_patches, apply_patches_case_sensitive, generate_patches,
    generate_patches_case_sensitive, PatchError,
};
pub use pointer::{find_pointer_from_object_to, get_pointer, get_pointer_case_sensitive};
pub use sort::{sort_object, sort_object_case_sensitive};
