use super::colour::Colour;
use super::content_encoding::ContentEncoding;
use super::projection::Projection;
use super::track::Track;
use super::tracks::kAv1CodecId;
use super::util;
use super::writer::Writer;
use crate::MkvId;

use std::ops::Deref;

// Supported modes for stereo 3D.
enum StereoMode {
    kMono = 0,
    kSideBySideLeftIsFirst = 1,
    kTopBottomRightIsFirst = 2,
    kTopBottomLeftIsFirst = 3,
    kSideBySideRightIsFirst = 11,
}

enum AlphaMode {
    kNoAlpha = 0,
    kAlpha = 1,
}

pub struct VideoTrack {
    track_: Track,

    // Video track element names.
    display_height_: u64,
    display_width_: u64,
    pixel_height_: u64,
    pixel_width_: u64,
    crop_left_: u64,
    crop_right_: u64,
    crop_top_: u64,
    crop_bottom_: u64,
    frame_rate_: f64,
    height_: u64,
    stereo_mode_: u64,
    alpha_mode_: u64,
    width_: u64,
    colour_space_: String,

    colour_: Option<Colour>,
    projection_: Option<Projection>,
}

impl Deref for VideoTrack {
    type Target = Track;

    fn deref(&self) -> &Track {
        &self.track_
    }
}

impl VideoTrack {
    pub fn new() -> VideoTrack {
        VideoTrack {
            track_: Track::new(),
            display_height_: 0,
            display_width_: 0,
            pixel_height_: 0,
            pixel_width_: 0,
            crop_left_: 0,
            crop_right_: 0,
            crop_top_: 0,
            crop_bottom_: 0,
            frame_rate_: 0.0,
            height_: 0,
            stereo_mode_: 0,
            alpha_mode_: 0,
            width_: 0,
            colour_space_: String::new(),
            colour_: None,
            projection_: None,
        }
    }

    // Sets the video's stereo mode. Returns true on success.
    pub fn SetStereoMode(&mut self, stereo_mode: u64) -> bool {
        if stereo_mode != StereoMode::kMono as u64
            && stereo_mode != StereoMode::kSideBySideLeftIsFirst as u64
            && stereo_mode != StereoMode::kTopBottomRightIsFirst as u64
            && stereo_mode != StereoMode::kTopBottomLeftIsFirst as u64
            && stereo_mode != StereoMode::kSideBySideRightIsFirst as u64
        {
            return false;
        }

        self.stereo_mode_ = stereo_mode;
        true
    }

    // Sets the video's alpha mode. Returns true on success.
    pub fn SetAlphaMode(&mut self, alpha_mode: u64) -> bool {
        if alpha_mode != AlphaMode::kNoAlpha as u64 && alpha_mode != AlphaMode::kAlpha as u64 {
            return false;
        }

        self.alpha_mode_ = alpha_mode;
        true
    }

    pub fn set_display_height(&mut self, height: u64) {
        self.display_height_ = height;
    }
    pub fn display_height(&self) -> u64 {
        return self.display_height_;
    }
    pub fn set_display_width(&mut self, width: u64) {
        self.display_width_ = width;
    }
    pub fn display_width(&self) -> u64 {
        return self.display_width_;
    }
    pub fn set_pixel_height(&mut self, height: u64) {
        self.pixel_height_ = height;
    }
    pub fn pixel_height(&self) -> u64 {
        return self.pixel_height_;
    }
    pub fn set_pixel_width(&mut self, width: u64) {
        self.pixel_width_ = width;
    }
    pub fn pixel_width(&self) -> u64 {
        return self.pixel_width_;
    }

    pub fn set_crop_left(&mut self, crop_left: u64) {
        self.crop_left_ = crop_left;
    }
    pub fn crop_left(&self) -> u64 {
        return self.crop_left_;
    }
    pub fn set_crop_right(&mut self, crop_right: u64) {
        self.crop_right_ = crop_right;
    }
    pub fn crop_right(&self) -> u64 {
        return self.crop_right_;
    }
    pub fn set_crop_top(&mut self, crop_top: u64) {
        self.crop_top_ = crop_top;
    }
    pub fn crop_top(&self) -> u64 {
        return self.crop_top_;
    }
    pub fn set_crop_bottom(&mut self, crop_bottom: u64) {
        self.crop_bottom_ = crop_bottom;
    }
    pub fn crop_bottom(&self) -> u64 {
        return self.crop_bottom_;
    }

    pub fn set_frame_rate(&mut self, frame_rate: f64) {
        self.frame_rate_ = frame_rate;
    }
    pub fn frame_rate(&self) -> f64 {
        return self.frame_rate_;
    }
    pub fn set_height(&mut self, height: u64) {
        self.height_ = height;
    }
    pub fn height(&self) -> u64 {
        return self.height_;
    }
    pub fn stereo_mode(&self) -> u64 {
        return self.stereo_mode_;
    }
    pub fn alpha_mode(&self) -> u64 {
        return self.alpha_mode_;
    }
    pub fn set_width(&mut self, width: u64) {
        self.width_ = width;
    }
    pub fn width(&self) -> u64 {
        return self.width_;
    }
    pub fn set_colour_space(&mut self, colour_space: &str) {
        self.colour_space_ = colour_space.to_string();
    }
    pub fn colour_space(&self) -> &str {
        return &self.colour_space_;
    }

    pub fn colour(&self) -> Option<&Colour> {
        return self.colour_.as_ref();
    }

    // Deep copies |colour|.
    pub fn SetColour(&mut self, colour: &Colour) {
        let mut colour_ = Colour::new();
        if let Some(mastering_metadata) = colour.mastering_metadata() {
            colour_.set_mastering_metadata(mastering_metadata);
        }

        colour_.set_matrix_coefficients(colour.matrix_coefficients());
        colour_.set_bits_per_channel(colour.bits_per_channel());
        colour_.set_chroma_subsampling_horz(colour.chroma_subsampling_horz());
        colour_.set_chroma_subsampling_vert(colour.chroma_subsampling_vert());
        colour_.set_cb_subsampling_horz(colour.cb_subsampling_horz());
        colour_.set_cb_subsampling_vert(colour.cb_subsampling_vert());
        colour_.set_chroma_siting_horz(colour.chroma_siting_horz());
        colour_.set_chroma_siting_vert(colour.chroma_siting_vert());
        colour_.set_range(colour.range());
        colour_.set_transfer_characteristics(colour.transfer_characteristics());
        colour_.set_primaries(colour.primaries());
        colour_.set_max_cll(colour.max_cll());
        colour_.set_max_fall(colour.max_fall());
        self.colour_ = Some(colour_);
    }

    pub fn projection(&self) -> Option<&Projection> {
        return self.projection_.as_ref();
    }

    // Deep copies |projection|.
    pub fn SetProjection(&mut self, projection: &Projection) {
        let mut projection_ = Projection::new();
        if !projection.private_data().is_empty() {
            projection_.set_private_data(projection.private_data())
        }
        projection_.set_type(projection.project_type());
        projection_.set_pose_yaw(projection.pose_yaw());
        projection_.set_pose_pitch(projection.pose_pitch());
        projection_.set_pose_roll(projection.pose_roll());
        self.projection_ = Some(projection_)
    }

    pub fn VideoPayloadSize(&self) -> u64 {
        let mut size = util::EbmlElementSizeArgU64(
            MkvId::MkvPixelWidth,
            if self.pixel_width_ > 0 {
                self.pixel_width_
            } else {
                self.width_
            },
        );

        size += util::EbmlElementSizeArgU64(
            MkvId::MkvPixelHeight,
            if self.pixel_height_ > 0 {
                self.pixel_height_
            } else {
                self.height_
            },
        );
        if self.display_width_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvDisplayWidth, self.display_width_);
        }
        if self.display_height_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvDisplayHeight, self.display_height_);
        }
        if self.crop_left_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvPixelCropLeft, self.crop_left_);
        }
        if self.crop_right_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvPixelCropRight, self.crop_right_);
        }
        if self.crop_top_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvPixelCropTop, self.crop_top_);
        }
        if self.crop_bottom_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvPixelCropBottom, self.crop_bottom_);
        }
        if self.stereo_mode_ > StereoMode::kMono as u64 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvStereoMode, self.stereo_mode_);
        }
        if self.alpha_mode_ > AlphaMode::kNoAlpha as u64 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvAlphaMode, self.alpha_mode_);
        }
        if self.frame_rate_ > 0.0 {
            size += util::EbmlElementSizeArgF32(MkvId::MkvFrameRate, self.frame_rate_ as f32);
        }
        if !self.colour_space_.is_empty() {
            size += util::EbmlElementSizeArgStr(MkvId::MkvColourSpace, &self.colour_space_);
        }
        if let Some(c) = self.colour_.as_ref() {
            size += c.Size();
        }
        if let Some(p) = self.projection_.as_ref() {
            size += p.Size();
        }

        return size;
    }

    pub fn PayloadSize(&self) -> u64 {
        let parent_size = self.track_.PayloadSize();

        let mut size = self.VideoPayloadSize();
        size += util::EbmlMasterElementSize(MkvId::MkvVideo, size);

        parent_size + size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        if !self.track_.Write(writer) {
            return false;
        }

        let size = self.VideoPayloadSize();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvVideo, size) {
            return false;
        }

        let payload_position = writer.get_position();

        if !util::WriteEbmlElementArgU64(
            writer,
            MkvId::MkvPixelWidth,
            if self.pixel_width_ > 0 {
                self.pixel_width_
            } else {
                self.width_
            },
        ) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(
            writer,
            MkvId::MkvPixelHeight,
            if self.pixel_height_ > 0 {
                self.pixel_height_
            } else {
                self.height_
            },
        ) {
            return false;
        }
        if self.display_width_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvDisplayWidth, self.display_width_) {
                return false;
            }
        }
        if self.display_height_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvDisplayHeight, self.display_height_)
            {
                return false;
            }
        }
        if self.crop_left_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvPixelCropLeft, self.crop_left_) {
                return false;
            }
        }
        if self.crop_right_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvPixelCropRight, self.crop_right_) {
                return false;
            }
        }
        if self.crop_top_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvPixelCropTop, self.crop_top_) {
                return false;
            }
        }
        if self.crop_bottom_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvPixelCropBottom, self.crop_bottom_) {
                return false;
            }
        }
        if self.stereo_mode_ > StereoMode::kMono as u64 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvStereoMode, self.stereo_mode_) {
                return false;
            }
        }
        if self.alpha_mode_ > AlphaMode::kNoAlpha as u64 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvAlphaMode, self.alpha_mode_) {
                return false;
            }
        }
        if !self.colour_space_.is_empty() {
            if !util::WriteEbmlElementArgStr(writer, MkvId::MkvColourSpace, &self.colour_space_) {
                return false;
            }
        }
        if self.frame_rate_ > 0.0 {
            if !util::WriteEbmlElementArgF32(writer, MkvId::MkvFrameRate, self.frame_rate_ as f32) {
                return false;
            }
        }
        if let Some(c) = self.colour_.as_ref() {
            if !c.Write(writer) {
                return false;
            }
        }
        if let Some(p) = self.projection_.as_ref() {
            if !p.Write(writer) {
                return false;
            }
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != size {
            return false;
        }

        true
    }
}
