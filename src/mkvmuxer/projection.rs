use super::util;
use super::writer::Writer;
use crate::MkvId;

#[derive(Debug, Copy, Clone)]
pub enum ProjectionType {
    kTypeNotPresent = -1,
    kRectangular = 0,
    kEquirectangular = 1,
    kCubeMap = 2,
    kMesh = 3,
}

pub struct Projection {
    type_: ProjectionType,
    pose_yaw_: f32,
    pose_pitch_: f32,
    pose_roll_: f32,
    private_data_: Vec<u8>,
}

impl Projection {
    const kValueNotPresent: u64 = std::u64::MAX;

    pub fn new() -> Projection {
        Projection {
            type_: ProjectionType::kRectangular,
            pose_yaw_: 0.0,
            pose_pitch_: 0.0,
            pose_roll_: 0.0,
            private_data_: Vec::new(),
        }
    }

    pub fn project_type(&self) -> ProjectionType {
        self.type_
    }
    pub fn set_type(&mut self, t: ProjectionType) {
        self.type_ = t;
    }
    pub fn pose_yaw(&self) -> f32 {
        self.pose_yaw_
    }
    pub fn set_pose_yaw(&mut self, pose_yaw: f32) {
        self.pose_yaw_ = pose_yaw;
    }
    pub fn pose_pitch(&self) -> f32 {
        self.pose_pitch_
    }
    pub fn set_pose_pitch(&mut self, pose_pitch: f32) {
        self.pose_pitch_ = pose_pitch;
    }
    pub fn pose_roll(&self) -> f32 {
        self.pose_roll_
    }
    pub fn set_pose_roll(&mut self, pose_roll: f32) {
        self.pose_roll_ = pose_roll;
    }
    pub fn private_data(&self) -> &[u8] {
        &self.private_data_
    }
    pub fn set_private_data(&mut self, data: &[u8]) {
        self.private_data_ = data.to_vec();
    }

    pub fn Size(&self) -> u64 {
        let mut size = self.PayloadSize();
        if size > 0 {
            size += util::EbmlMasterElementSize(MkvId::MkvProjection, size);
        }
        size
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut size = util::EbmlElementSizeArgU64(MkvId::MkvProjection, self.type_ as u64);

        if self.private_data_.len() > 0 {
            size += util::EbmlElementSizeArgSlice(MkvId::MkvProjectionPrivate, &self.private_data_);
        }

        size += util::EbmlElementSizeArgF32(MkvId::MkvProjectionPoseYaw, self.pose_yaw_);
        size += util::EbmlElementSizeArgF32(MkvId::MkvProjectionPosePitch, self.pose_pitch_);
        size += util::EbmlElementSizeArgF32(MkvId::MkvProjectionPoseRoll, self.pose_roll_);

        size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let size = self.PayloadSize();

        // Don't write an empty element.
        if size == 0 {
            return true;
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvProjection, size) {
            return false;
        }

        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvProjectionType, self.type_ as u64) {
            return false;
        }

        if self.private_data_.len() > 0
            && !util::WriteEbmlElementArgSlice(
                writer,
                MkvId::MkvProjectionPrivate,
                &self.private_data_,
            )
        {
            return false;
        }

        if !util::WriteEbmlElementArgF32(writer, MkvId::MkvProjectionPoseYaw, self.pose_yaw_) {
            return false;
        }

        if !util::WriteEbmlElementArgF32(writer, MkvId::MkvProjectionPosePitch, self.pose_pitch_) {
            return false;
        }

        if !util::WriteEbmlElementArgF32(writer, MkvId::MkvProjectionPoseRoll, self.pose_roll_) {
            return false;
        }

        true
    }
}
