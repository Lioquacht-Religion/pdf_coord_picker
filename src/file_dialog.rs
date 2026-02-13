// file_dialog.rs

#[cfg(not(target_arch = "wasm32"))]
pub mod file_dialog_native;

#[cfg(target_arch = "wasm32")]
pub mod file_dialog_web;
