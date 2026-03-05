use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, Write};

use crate::common::{C3Vector, M2Array};
use crate::error::Result;
use crate::version::M2Version;

bitflags::bitflags! {
    /// Particle flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2ParticleFlags: u32 {
        /// Particle has a position
        const HAS_POSITION = 0x00000001;
        /// Particles are billboarded
        const BILLBOARDED = 0x00000008;
        /// Particles stretch based on their velocity
        const AFFECTED_BY_VELOCITY = 0x00000010;
        /// Particles rotate around their central point
        const ROTATING = 0x00000020;
        /// Particles use random texture coordinate generation
        const RANDOMIZED = 0x00000040;
        /// Particles use tiling
        const TILED = 0x00000080;
        /// ModelParticleEmitterType::Plane should be treated as ModelParticleEmitterType::Sphere
        const SPHERE_AS_SOURCE = 0x00000100;
        /// The center of the sphere should be used as the source of the particles
        const USE_SPHERE_CENTER = 0x00000200;
        /// Enable lighting for particles
        const LIGHTING = 0x00000400;
        /// Use a Z-buffer test for particles
        const ZBUFFER_TEST = 0x00000800;
        /// Use particle bounds for culling
        const BOUND_TO_EMITTER = 0x00001000;
        /// Particles follow their emitter
        const FOLLOW_EMITTER = 0x00002000;
        /// Unknown, used in the Deeprun Tram subway
        const UNKNOWN_0x4000 = 0x00004000;
        /// Unknown, used in the Deeprun Tram subway
        const UNKNOWN_0x8000 = 0x00008000;
        /// Unknown, used in the character display window
        const UNKNOWN_0x10000 = 0x00010000;
        /// Random spawn position
        const RANDOM_SPAWN_POSITION = 0x00020000;
        /// Particles stretch based on particle size
        const PINNED = 0x00040000;
        /// Use XYZ rotation instead of just Z
        const XYZ_ROTATION = 0x00080000;
        /// Unknown, was added in WoD (6.x)
        const UNKNOWN_WOD = 0x00100000;
        /// Use physics settings for particles
        const PHYSICS = 0x00200000;
        /// Pinned on the Y axis instead of both XY
        const FIXED_Y = 0x00400000;
    }
}

/// Particle emitter type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2ParticleEmitterType {
    /// Point emitter (particles spawn from a single point)
    Point = 0,
    /// Plane emitter (particles spawn within a 2D plane)
    Plane = 1,
    /// Sphere emitter (particles spawn within a 3D sphere)
    Sphere = 2,
    /// Spline emitter (particles follow a spline path)
    Spline = 3,
    /// Bone emitter (particles spawn from a bone)
    Bone = 4,
}

impl M2ParticleEmitterType {
    /// Parse from integer value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Point),
            1 => Some(Self::Plane),
            2 => Some(Self::Sphere),
            3 => Some(Self::Spline),
            4 => Some(Self::Bone),
            _ => None,
        }
    }
}

/// Represents a particle emitter in an M2 model
#[derive(Debug, Clone)]
pub struct M2ParticleEmitter {
    /// ID for this emitter
    pub id: u32,
    /// Flags controlling particle behavior
    pub flags: M2ParticleFlags,
    /// Position of the emitter
    pub position: C3Vector,
    /// Bone to attach the emitter to
    pub bone_index: u16,
    /// Texture coordinate (for UV coordinate generation)
    pub texture_index: u16,
    /// Geometry model filename (for complex shaped emitters)
    pub model_filename: M2Array<u8>,
    /// Explicit fallback model if main one fails to load
    pub fallback_model_filename: M2Array<u8>,
    /// Blending type
    pub blending_type: u8,
    /// Emitter type
    pub emitter_type: M2ParticleEmitterType,
    /// Particle type
    pub particle_type: u8,
    /// Head or tail
    pub head_or_tail: u8,
    /// Texture file IDs (for multi-texture particles)
    pub texture_file_data_ids: Option<M2Array<u32>>,
    /// Flag to enable encryption (WoD and later)
    pub enable_encryption: Option<u8>,
    /// Multi-texture particle blend operation
    pub multi_texture_param0: Option<[u8; 4]>,
    /// Multi-texture particle blend flags
    pub multi_texture_param1: Option<[u8; 4]>,
    /// Texture tile rotation
    pub texture_tile_rotation: u16,
    /// Texture dimensions for tiling
    pub texture_dimensions_rows: u16,
    pub texture_dimensions_columns: u16,
}

impl M2ParticleEmitter {
    /// Parse a particle emitter from a reader based on the M2 version
    pub fn parse<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
        let id = reader.read_u32_le()?;
        let flag_bits = reader.read_u32_le()?;
        let flags = M2ParticleFlags::from_bits_retain(flag_bits);
        let position = if flags.contains(M2ParticleFlags::HAS_POSITION) {
            C3Vector::parse(reader)?
        } else {
            C3Vector::default()
        };
        let bone_index = reader.read_u16_le()?;
        let texture_index = reader.read_u16_le()?;
        let model_filename = M2Array::parse(reader)?;
        let fallback_model_filename = M2Array::<u8>::parse(reader)?;

        // Version-specific fields
        let (
            blending_type,
            emitter_type,
            particle_type,
            head_or_tail,
            texture_file_data_ids,
            enable_encryption,
            multi_texture_param0,
            multi_texture_param1,
        ) = if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version >= M2Version::Legion {
                let blend = reader.read_u8()?;
                let emitter = M2ParticleEmitterType::from_u8(reader.read_u8()?)
                    .unwrap_or(M2ParticleEmitterType::Point);
                let particle = reader.read_u8()?;
                let head = reader.read_u8()?;
                let tex_file_ids = M2Array::parse(reader)?;

                // Extra fields for WoD and later
                let encryption = if m2_version >= M2Version::WoD {
                    Some(reader.read_u8()?)
                } else {
                    None
                };

                // Extra multi-texture params for BfA and later
                let (param0, param1) = if m2_version >= M2Version::BfA {
                    let mut p0 = [0u8; 4];
                    let mut p1 = [0u8; 4];

                    for item in &mut p0 {
                        *item = reader.read_u8()?;
                    }

                    for item in &mut p1 {
                        *item = reader.read_u8()?;
                    }

                    (Some(p0), Some(p1))
                } else {
                    (None, None)
                };

                (
                    blend,
                    emitter,
                    particle,
                    head,
                    Some(tex_file_ids),
                    encryption,
                    param0,
                    param1,
                )
            } else if m2_version >= M2Version::WoD {
                // WoD has encryption but no fallback model or texture file IDs
                let blend = reader.read_u8()?;
                let emitter = M2ParticleEmitterType::from_u8(reader.read_u8()?)
                    .unwrap_or(M2ParticleEmitterType::Point);
                let particle = reader.read_u8()?;
                let head = reader.read_u8()?;
                let encryption = Some(reader.read_u8()?);

                (blend, emitter, particle, head, None, encryption, None, None)
            } else {
                // Pre-WoD just has basic fields
                let blend = reader.read_u16_le()? as u8;
                let emitter = M2ParticleEmitterType::Point;
                M2ParticleEmitterType::from_u8(reader.read_u16_le()? as u8)
                    .unwrap_or(M2ParticleEmitterType::Point);
                let particle = reader.read_u8()?;
                let head = reader.read_u8()?;

                (blend, emitter, particle, head, None, None, None, None)
            }
        } else {
            // Default to Vanilla format
            let blend = reader.read_u16_le()? as u8;
            let emitter = M2ParticleEmitterType::Point;
            M2ParticleEmitterType::from_u8(reader.read_u16_le()? as u8)
                .unwrap_or(M2ParticleEmitterType::Point);
            let particle = reader.read_u8()?;
            let head = reader.read_u8()?;

            (blend, emitter, particle, head, None, None, None, None)
        };

        let texture_tile_rotation = reader.read_u16_le()?;
        let texture_dimensions_rows = reader.read_u16_le()?;
        let texture_dimensions_columns = reader.read_u16_le()?;

        Ok(Self {
            id,
            flags,
            position,
            bone_index,
            texture_index,
            model_filename,
            fallback_model_filename,
            blending_type,
            emitter_type,
            particle_type,
            head_or_tail,
            texture_file_data_ids,
            enable_encryption,
            multi_texture_param0,
            multi_texture_param1,
            texture_tile_rotation,
            texture_dimensions_rows,
            texture_dimensions_columns,
        })
    }

    /// Write a particle emitter to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {
        writer.write_u32_le(self.id)?;
        writer.write_u32_le(self.flags.bits())?;
        self.position.write(writer)?;
        writer.write_u16_le(self.bone_index)?;
        writer.write_u16_le(self.texture_index)?;
        self.model_filename.write(writer)?;
        self.fallback_model_filename.write(writer)?;

        // Version-specific fields
        if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version >= M2Version::Legion {
                writer.write_u8(self.blending_type)?;
                writer.write_u8(self.emitter_type as u8)?;
                writer.write_u8(self.particle_type)?;
                writer.write_u8(self.head_or_tail)?;

                if let Some(ref tex_file_ids) = self.texture_file_data_ids {
                    tex_file_ids.write(writer)?;
                } else {
                    M2Array::<u32>::new(0, 0).write(writer)?;
                }

                // Extra fields for WoD and later
                if m2_version >= M2Version::WoD {
                    writer.write_u8(self.enable_encryption.unwrap_or(0))?;
                }

                // Extra multi-texture params for BfA and later
                if m2_version >= M2Version::BfA {
                    if let Some(param0) = self.multi_texture_param0 {
                        for &val in &param0 {
                            writer.write_u8(val)?;
                        }
                    } else {
                        for _ in 0..4 {
                            writer.write_u8(0)?;
                        }
                    }

                    if let Some(param1) = self.multi_texture_param1 {
                        for &val in &param1 {
                            writer.write_u8(val)?;
                        }
                    } else {
                        for _ in 0..4 {
                            writer.write_u8(0)?;
                        }
                    }
                }
            } else if m2_version >= M2Version::WoD {
                // WoD has encryption but no fallback model or texture file IDs
                writer.write_u8(self.blending_type)?;
                writer.write_u8(self.emitter_type as u8)?;
                writer.write_u8(self.particle_type)?;
                writer.write_u8(self.head_or_tail)?;
                writer.write_u8(self.enable_encryption.unwrap_or(0))?;
            } else {
                // Pre-WoD just has basic fields
                writer.write_u8(self.blending_type)?;
                writer.write_u8(self.emitter_type as u8)?;
                writer.write_u8(self.particle_type)?;
                writer.write_u8(self.head_or_tail)?;
            }
        } else {
            // Default to Vanilla format
            writer.write_u8(self.blending_type)?;
            writer.write_u8(self.emitter_type as u8)?;
            writer.write_u8(self.particle_type)?;
            writer.write_u8(self.head_or_tail)?;
        }

        // Texture tile coordinates are in all versions
        writer.write_u16_le(self.texture_tile_rotation)?;
        writer.write_u16_le(self.texture_dimensions_rows)?;
        writer.write_u16_le(self.texture_dimensions_columns)?;

        Ok(())
    }

    /// Convert this particle emitter to a different version
    pub fn convert(&self, target_version: M2Version) -> Self {
        let mut new_emitter = self.clone();

        // Handle version-specific conversions
        if target_version >= M2Version::Legion && self.texture_file_data_ids.is_none() {
            // When upgrading to Legion or later, add fallback model filename and texture file data IDs if missing
            new_emitter.texture_file_data_ids = Some(M2Array::new(0, 0));
        } else if target_version < M2Version::Legion {
            // When downgrading to pre-Legion, remove fallback model filename and texture file data IDs
            new_emitter.texture_file_data_ids = None;
        }

        if target_version >= M2Version::WoD && self.enable_encryption.is_none() {
            // When upgrading to WoD or later, add encryption if missing
            new_emitter.enable_encryption = Some(0);
        } else if target_version < M2Version::WoD {
            // When downgrading to pre-WoD, remove encryption
            new_emitter.enable_encryption = None;
        }

        if target_version >= M2Version::BfA && self.multi_texture_param0.is_none() {
            // When upgrading to BfA or later, add multi-texture params if missing
            new_emitter.multi_texture_param0 = Some([0, 0, 0, 0]);
            new_emitter.multi_texture_param1 = Some([0, 0, 0, 0]);
        } else if target_version < M2Version::BfA {
            // When downgrading to pre-BfA, remove multi-texture params
            new_emitter.multi_texture_param0 = None;
            new_emitter.multi_texture_param1 = None;
        }

        new_emitter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_emitter_flags() {
        let flags = M2ParticleFlags::BILLBOARDED | M2ParticleFlags::ROTATING;
        assert!(flags.contains(M2ParticleFlags::BILLBOARDED));
        assert!(flags.contains(M2ParticleFlags::ROTATING));
        assert!(!flags.contains(M2ParticleFlags::PHYSICS));
    }
}
