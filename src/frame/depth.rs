//! Type for representing a depth frame taken from a depth camera.

use super::frame_traits::{
    FrameConstructionError, PixelIndexOutOfBoundsError, VideoFrameEx, VideoFrameUnsafeEx,
};
use super::{iter::ImageIter, kind::Kind};
use crate::{common::*, stream};
use std::ffi::CStr;
use std::result::Result;

pub struct DepthFrame<'a> {
    frame_ptr: NonNull<sys::rs2_frame>,
    width: usize,
    height: usize,
    stride: usize,
    bits_per_pixel: usize,
    frame_stream_profile: stream::Profile,
    data: &'a [u16],
}

impl<'a> DepthFrame<'a> {
    fn profile(&'a self) -> &'a stream::Profile {
        &self.frame_stream_profile
    }

    fn iter(&'a self) -> ImageIter<'a, DepthFrame<'a>> {
        ImageIter {
            frame: self,
            column: 0,
            row: 0,
        }
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

impl<'a> std::convert::TryFrom<NonNull<sys::rs2_frame>> for DepthFrame<'a> {
    type Error = FrameConstructionError;

    fn try_from(frame_ptr: NonNull<sys::rs2_frame>) -> Result<Self, Self::Error> {
        unsafe {
            let mut err: *mut sys::rs2_error = ptr::null_mut();
            let width = sys::rs2_get_frame_width(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(FrameConstructionError::CouldNotGetWidth(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let height = sys::rs2_get_frame_height(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(FrameConstructionError::CouldNotGetHeight(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let bits_per_pixel = sys::rs2_get_frame_bits_per_pixel(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(FrameConstructionError::CouldNotGetBitsPerPixel(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let stride = sys::rs2_get_frame_stride_in_bytes(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(FrameConstructionError::CouldNotGetStride(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }

            let profile_ptr = sys::rs2_get_frame_stream_profile(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(FrameConstructionError::CouldNotGetFrameStreamProfile(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }
            let nonnull_profile_ptr =
                NonNull::new(profile_ptr as *mut sys::rs2_stream_profile).unwrap();
            let profile = stream::Profile::new(nonnull_profile_ptr).map_err(|_| {
                FrameConstructionError::CouldNotGetFrameStreamProfile(String::from(
                    "Could not construct stream profile.",
                ))
            })?;

            let size = sys::rs2_get_frame_data_size(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(FrameConstructionError::CouldNotGetDataSize(
                    CStr::from_ptr(sys::rs2_get_error_message(err))
                        .to_str()
                        .unwrap()
                        .to_string(),
                ));
            }

            let ptr = sys::rs2_get_frame_data(frame_ptr.as_ptr(), &mut err);
            if NonNull::new(err).is_some() {
                return Err(FrameConstructionError::CouldNotGetData(
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

impl<'a> VideoFrameUnsafeEx for DepthFrame<'a> {
    type Output = &'a u16;

    fn at_no_bounds_check(&self, col: usize, row: usize) -> Self::Output {
        let offset = row * self.width + col;
        &self.data[offset]
    }
}

impl<'a> VideoFrameEx for DepthFrame<'a> {
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

    fn at(&self, col: usize, row: usize) -> Result<Self::Output, PixelIndexOutOfBoundsError> {
        if col >= self.width || row >= self.height {
            Err(PixelIndexOutOfBoundsError())
        } else {
            Ok(self.at_no_bounds_check(col, row))
        }
    }
}

impl<'a> IntoIterator for &'a DepthFrame<'a> {
    type Item = <ImageIter<'a, DepthFrame<'a>> as Iterator>::Item;
    type IntoIter = ImageIter<'a, DepthFrame<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
