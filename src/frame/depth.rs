use super::frame_trait::{ConstructionError, Frame};
use super::kind::Kind;
use crate::{common::*, stream};
use std::ffi::CStr;

struct DepthFrame<'a> {
    frame_ptr: NonNull<sys::rs2_frame>,
    width: usize,
    height: usize,
    stride: usize,
    bits_per_pixel: usize,
    frame_stream_profile: stream::Profile,
    data: &'a [u16],
}

impl<'a> DepthFrame<'a> {
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }

    fn stride(&self) -> usize {
        self.stride
    }

    fn bits_per_pixel(&self) -> usize {
        self.bits_per_pixel
    }
}

impl<'a> Drop for DepthFrame<'a> {
    fn drop(&mut self) {
        unsafe {
            sys::rs2_release_frame(self.frame_ptr.as_ptr());
        }
    }
}

impl<'a> Kind for DepthFrame<'a> {
    fn extension() -> sys::rs2_extension {
        sys::rs2_extension_RS2_EXTENSION_DEPTH_FRAME
    }
}

impl<'a> Frame for DepthFrame<'a>
where
    Self: Sized,
{
    fn new(frame_ptr: NonNull<sys::rs2_frame>) -> std::result::Result<Self, ConstructionError> {
        unsafe {
            let mut err: *mut sys::rs2_error = ptr::null_mut();
            let width = sys::rs2_get_frame_width(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(ConstructionError::CouldNotGetWidth(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let height = sys::rs2_get_frame_height(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(ConstructionError::CouldNotGetHeight(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let bits_per_pixel = sys::rs2_get_frame_bits_per_pixel(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(ConstructionError::CouldNotGetBitsPerPixel(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let stride = sys::rs2_get_frame_stride_in_bytes(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(ConstructionError::CouldNotGetStride(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }

            let profile_ptr = sys::rs2_get_frame_stream_profile(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(ConstructionError::CouldNotGetFrameStreamProfile(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let nonnull_profile_ptr =
                NonNull::new(profile_ptr as *mut sys::rs2_stream_profile).unwrap();
            let profile = stream::Profile::new(nonnull_profile_ptr).map_err(|e| {
                ConstructionError::CouldNotGetFrameStreamProfile(String::from(
                    "Could not construct stream profile.",
                ))
            })?;

            let size = sys::rs2_get_frame_data_size(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(ConstructionError::CouldNotGetDataSize(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }

            let ptr = sys::rs2_get_frame_data(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(ConstructionError::CouldNotGetData(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let data = slice::from_raw_parts(ptr.cast::<u16>(), size as usize);

            Ok(DepthFrame {
                frame_ptr,
                width: width as usize,
                height: height as usize,
                stride: stride as usize,
                bits_per_pixel: bits_per_pixel as usize,
                frame_stream_profile: profile,
                data,
            })
        }
    }
}
