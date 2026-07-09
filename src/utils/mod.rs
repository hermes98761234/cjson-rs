pub mod pointer;
pub mod patch;
pub mod merge_patch;
pub mod sort;

pub use pointer::{get_pointer, get_pointer_case_sensitive, find_pointer_from_object_to};
pub use patch::{
    apply_patches, apply_patches_case_sensitive, generate_patches,
    generate_patches_case_sensitive, add_patch_to_array, PatchError,
};
pub use merge_patch::{
    merge_patch, merge_patch_case_sensitive, generate_merge_patch,
    generate_merge_patch_case_sensitive,
};
pub use sort::{sort_object, sort_object_case_sensitive};
