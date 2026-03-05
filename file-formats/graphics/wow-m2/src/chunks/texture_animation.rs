use crate::chunks::C4Quaternion;
use crate::chunks::animation::M2AnimationBlock;
use crate::common::C3Vector;
use std::io::{Read, Seek, Write};

use crate::error::Result;
use crate::version::M2Version;

/// Texture animation structure
#[derive(Debug, Clone, Default)]
pub struct M2TextureAnimation {
    /// Animation for translation
    pub translation: M2AnimationBlock<C3Vector>, // M2TrackVec3,
    /// Rotation animation
    pub rotation: M2AnimationBlock<C4Quaternion>, // M2TrackQuat,
    /// Scale animation
    pub scale: M2AnimationBlock<C3Vector>, //M2TrackVec3,
}

impl M2TextureAnimation {
    /// Parse a texture animation from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let translation = M2AnimationBlock::parse(reader)?;
        let rotation = M2AnimationBlock::parse(reader)?;
        let scale = M2AnimationBlock::parse(reader)?;

        Ok(Self {
            translation,
            rotation,
            scale,
        })
    }

    /// Write a texture animation to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.translation.write(writer)?;
        self.rotation.write(writer)?;
        self.scale.write(writer)?;

        Ok(())
    }

    /// Convert this texture animation to a different version (no version differences yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }

    /// Create a new texture animation with default values
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_texture_animation_parse_write() {
        let mut data = Vec::new();

        // Translation animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset

        // Rotation animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset

        // Scale animation track
        data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
        data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges count
        data.extend_from_slice(&0u32.to_le_bytes()); // Interpolation ranges offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
        data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
        data.extend_from_slice(&0u32.to_le_bytes()); // Values count
        data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
    }
}
