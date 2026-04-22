use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;

use image::DynamicImage;

use crate::t_image::resize_dynamic_image_to_jpeg;

// Minimal libheif FFI for decoding the primary image to RGB.
// We keep this narrow to avoid pulling bindgen into the build.

#[repr(C)]
#[derive(Clone, Copy)]
struct HeifError {
    code: c_int,
    subcode: c_int,
    message: *const c_char,
}

#[repr(C)]
struct HeifContext(c_void);
#[repr(C)]
struct HeifImageHandle(c_void);
#[repr(C)]
struct HeifImage(c_void);

// heif_colorspace_RGB
const HEIF_COLORSPACE_RGB: c_int = 3;
// heif_chroma_interleaved_RGB
const HEIF_CHROMA_INTERLEAVED_RGB: c_int = 11;
// heif_channel_interleaved
const HEIF_CHANNEL_INTERLEAVED: c_int = 10;

extern "C" {
    fn heif_context_alloc() -> *mut HeifContext;
    fn heif_context_free(ctx: *mut HeifContext);
    fn heif_context_read_from_file(ctx: *mut HeifContext, filename: *const c_char, options: *const c_void) -> HeifError;
    fn heif_context_get_primary_image_handle(ctx: *mut HeifContext, handle: *mut *mut HeifImageHandle) -> HeifError;
    fn heif_image_handle_release(handle: *mut HeifImageHandle);

    fn heif_decode_image(
        handle: *const HeifImageHandle,
        out_img: *mut *mut HeifImage,
        colorspace: c_int,
        chroma: c_int,
        options: *const c_void,
    ) -> HeifError;
    fn heif_image_release(img: *mut HeifImage);

    fn heif_image_get_width(img: *const HeifImage, channel: c_int) -> c_int;
    fn heif_image_get_height(img: *const HeifImage, channel: c_int) -> c_int;
    fn heif_image_get_plane_readonly(img: *const HeifImage, channel: c_int, out_stride: *mut c_int) -> *const u8;
}

fn fmt_heif_error(err: HeifError) -> String {
    if err.code == 0 {
        return "ok".to_string();
    }
    unsafe {
        let msg = if err.message.is_null() {
            ""
        } else {
            std::ffi::CStr::from_ptr(err.message).to_string_lossy().as_ref()
        };
        format!("libheif error code={} subcode={} msg={}", err.code, err.subcode, msg)
    }
}

fn decode_primary_rgb(file_path: &str) -> Result<(Vec<u8>, u32, u32, u32), String> {
    let c_path = CString::new(file_path).map_err(|_| "Invalid file path".to_string())?;
    unsafe {
        let ctx = heif_context_alloc();
        if ctx.is_null() {
            return Err("Failed to allocate heif context".to_string());
        }
        struct CtxGuard(*mut HeifContext);
        impl Drop for CtxGuard {
            fn drop(&mut self) {
                unsafe { heif_context_free(self.0) }
            }
        }
        let _ctx_guard = CtxGuard(ctx);

        let err = heif_context_read_from_file(ctx, c_path.as_ptr(), ptr::null());
        if err.code != 0 {
            return Err(fmt_heif_error(err));
        }

        let mut handle: *mut HeifImageHandle = ptr::null_mut();
        let err = heif_context_get_primary_image_handle(ctx, &mut handle);
        if err.code != 0 || handle.is_null() {
            return Err(fmt_heif_error(err));
        }
        struct HandleGuard(*mut HeifImageHandle);
        impl Drop for HandleGuard {
            fn drop(&mut self) {
                unsafe { heif_image_handle_release(self.0) }
            }
        }
        let _handle_guard = HandleGuard(handle);

        let mut img: *mut HeifImage = ptr::null_mut();
        let err = heif_decode_image(handle, &mut img, HEIF_COLORSPACE_RGB, HEIF_CHROMA_INTERLEAVED_RGB, ptr::null());
        if err.code != 0 || img.is_null() {
            return Err(fmt_heif_error(err));
        }
        struct ImgGuard(*mut HeifImage);
        impl Drop for ImgGuard {
            fn drop(&mut self) {
                unsafe { heif_image_release(self.0) }
            }
        }
        let _img_guard = ImgGuard(img);

        let width = heif_image_get_width(img, HEIF_CHANNEL_INTERLEAVED).max(0) as u32;
        let height = heif_image_get_height(img, HEIF_CHANNEL_INTERLEAVED).max(0) as u32;
        if width == 0 || height == 0 {
            return Err("libheif returned empty dimensions".to_string());
        }

        let mut stride: c_int = 0;
        let ptr_plane = heif_image_get_plane_readonly(img, HEIF_CHANNEL_INTERLEAVED, &mut stride);
        if ptr_plane.is_null() || stride <= 0 {
            return Err("libheif returned empty plane".to_string());
        }

        let stride_u = stride as u32;
        let row_bytes = width.saturating_mul(3);
        if stride_u < row_bytes {
            return Err("libheif returned invalid stride".to_string());
        }

        let src = slice::from_raw_parts(ptr_plane, (stride_u * height) as usize);
        let mut out = vec![0u8; (width * height * 3) as usize];
        for y in 0..height {
            let src_off = (y * stride_u) as usize;
            let dst_off = (y * row_bytes) as usize;
            out[dst_off..dst_off + row_bytes as usize]
                .copy_from_slice(&src[src_off..src_off + row_bytes as usize]);
        }

        Ok((out, width, height, row_bytes))
    }
}

pub fn get_heif_thumbnail(file_path: &str, orientation: i32, thumbnail_size: u32) -> Result<Option<Vec<u8>>, String> {
    let (rgb, width, height, _row_bytes) = decode_primary_rgb(file_path)?;
    // Build a DynamicImage to reuse existing orientation + alpha handling logic.
    // libheif decode gives us RGB, no alpha here.
    let img = image::RgbImage::from_raw(width, height, rgb)
        .ok_or_else(|| "Failed to build RGB image from libheif buffer".to_string())?;
    let dyn_img = DynamicImage::ImageRgb8(img);
    // Reuse existing pipeline for consistent orientation behavior.
    resize_dynamic_image_to_jpeg(dyn_img, orientation, thumbnail_size).map(Some)
}

pub fn get_heif_preview(file_path: &str, orientation: i32, max_size: u32) -> Result<Option<Vec<u8>>, String> {
    let (rgb, width, height, _row_bytes) = decode_primary_rgb(file_path)?;
    let img = image::RgbImage::from_raw(width, height, rgb)
        .ok_or_else(|| "Failed to build RGB image from libheif buffer".to_string())?;
    // Preview path: keep it JPEG encoded at up to max_size (same as thumbnail sizing semantics).
    // This uses the same resizing logic as thumbnails.
    resize_dynamic_image_to_jpeg(DynamicImage::ImageRgb8(img), orientation, max_size).map(Some)
}
