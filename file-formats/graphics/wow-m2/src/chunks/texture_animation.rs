use std::io::{Read, Seek, Write};

use crate::chunks::animation::M2AnimationBlock;
use crate::chunks::C4Quaternion;
use crate::common::C3Vector;
use crate::error::Result;
use crate::version::M2Version;

/// Texture animation type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2TextureAnimationType {
    /// No animation
    None = 0,
    /// Scroll animation
    Scroll = 1,
    /// Rotate animation
    Rotate = 2,
    /// Scale animation
    Scale = 3,
    /// Key frame animation
    KeyFrame = 4,
}

impl M2TextureAnimationType {
    /// Parse from integer value
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Scroll),
            2 => Some(Self::Rotate),
            3 => Some(Self::Scale),
            4 => Some(Self::KeyFrame),
            _ => None,
        }
    }
}

/// Texture animation structure
#[derive(Debug, Clone, Default)]
pub struct M2TextureAnimation {
    /// Translation animation (3D Vector)
    pub translation: M2AnimationBlock<C3Vector>, 
    /// Rotation animation (Quaternion)
    pub rotation: M2AnimationBlock<C4Quaternion>, 
    /// Scale animation (3D Vector)
    pub scale: M2AnimationBlock<C3Vector>,
}

impl M2TextureAnimation {
    /// Parse animation from raw binary data
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // No more fake "animation_type" and "padding", directly read the 3 blocks!
        let translation = M2AnimationBlock::parse(reader)?;
        let rotation = M2AnimationBlock::parse(reader)?;
        let scale = M2AnimationBlock::parse(reader)?;

        Ok(Self {
            translation,
            rotation,
            scale,
        })
    }

    /// Writes the animation to binary
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.translation.write(writer)?;
        self.rotation.write(writer)?;
        self.scale.write(writer)?;
        Ok(())
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Convert this texture animation to a different version (no version differences yet)
    pub fn convert(&self, _target_version: M2Version) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_texture_animation_parse_write() {
        let mut data = Vec::new();

        // Helper to write an empty M2AnimationBlock (20 bytes: 2+2+4+4+4+4)
        let write_empty_block = |data: &mut Vec<u8>| {
            data.extend_from_slice(&1u16.to_le_bytes()); // Interpolation type (Linear)
            data.extend_from_slice(&(-1i16).to_le_bytes()); // Global sequence
            data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps count
            data.extend_from_slice(&0u32.to_le_bytes()); // Timestamps offset
            data.extend_from_slice(&0u32.to_le_bytes()); // Values count
            data.extend_from_slice(&0u32.to_le_bytes()); // Values offset
        };

        // Translation block (C3Vector)
        write_empty_block(&mut data);
        // Rotation block (C4Quaternion)
        write_empty_block(&mut data);
        // Scale block (C3Vector)
        write_empty_block(&mut data);

        let mut cursor = Cursor::new(data);
        let tex_anim = M2TextureAnimation::parse(&mut cursor).unwrap();

        // Test write round-trip: output size should match input
        let mut output = Vec::new();
        tex_anim.write(&mut output).unwrap();
        assert_eq!(output.len(), cursor.get_ref().len());
    }
}
