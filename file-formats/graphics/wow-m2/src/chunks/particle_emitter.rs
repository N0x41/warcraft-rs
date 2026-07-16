use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Seek, Write};

use crate::chunks::animation::{M2AnimationBlock, FakeAnimationBlock};
//use crate::chunks::color_animation::M2Color;
use crate::common::{C2Vector, C3Vector, M2Array};
use crate::error::Result;
use crate::version::M2Version;

bitflags::bitflags! {
    /// Particle flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2ParticleFlags: u32 {
        /// Identifies that this particle has a position.
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

/// Represents a particle emitter in an M2 model (Legacy version for Vanilla, TBC, and WotLK).
/// Extracted and aligned strictly to the <= 264 version specifications.
#[derive(Debug, Clone)]
pub struct M2ParticleEmitter {
    /// Unique identifier for this emitter (often -1).
    pub id: u32,
    /// Flags controlling particle behavior and rendering.
    pub flags: M2ParticleFlags,
    /// Position of the emitter relative to the attached bone.
    pub position: C3Vector,
    /// The bone index this emitter is attached to.
    pub bone_index: u16,
    
    /// Texture indices used by the particle.
    /// Cataclysm introduced multi-texturing via a bitfield, occupying these same 2 bytes.
    pub texture_indices: [u16; 3], 
    
    /// Geometry model filename. If given, this emitter spawns model particles.
    pub model_filename: M2Array<u8>,
    
    /// Child emitters model filename. Introduced in WotLK (264).
    /// If given, child emitters are obtained from this model and emitted as a trail per particle.
    pub child_emitter_filename: Option<M2Array<u8>>, 
    
    /// A blending type for the particle (e.g., 0: Opaque, 1: AlphaBlend, 2: Additive, etc.).
    pub blending_type: u8,
    /// The shape of the emitter (1 - Plane, 2 - Sphere, 3 - Spline, 4 - Bone).
    pub emitter_type: M2ParticleEmitterType,
    
    /// Used in conjunction with ParticleColor.dbc to alter the default color.
    /// Introduced in late The Burning Crusade (262).
    pub particle_color_index: Option<u16>, 
    
    /// Type of the particle.
    pub particle_type: u8,
    /// Determines billboard rendering behavior (0 - Head, 1 - Tail, 2 - Both).
    pub head_or_tail: u8,
    
    /// The rendering priority plane.
    pub priority_plane: i16, 
    /// Number of texture rows for tiled textures.
    pub texture_dimensions_rows: u16,
    /// Number of texture columns for tiled textures.
    pub texture_dimensions_columns: u16,

    // ==========================================
    // ANIMATIONS (M2Track)
    // ==========================================
    /// Base velocity at which particles are emitted.
    pub emission_speed: M2AnimationBlock<f32>,
    /// Random variation in particle emission speed (range: 0 to 1).
    pub speed_variation: M2AnimationBlock<f32>,
    /// Maximum polar angle of the initial velocity (longitude drifting).
    pub vertical_range: M2AnimationBlock<f32>,
    /// Maximum azimuth angle of the initial velocity (latitude drifting).
    pub horizontal_range: M2AnimationBlock<f32>,
    /// Gravity applied to the particles over time.
    pub gravity: M2AnimationBlock<f32>,
    /// Number of seconds each particle continues to be drawn after its creation.
    pub lifespan: M2AnimationBlock<f32>,
    
    /// An individual particle's lifespan is added to by (lifespanVariation * random(-1, 1)).
    /// Introduced in WotLK (264).
    pub lifespan_variation: Option<f32>, 
    
    /// Emission rate of the particles (bursts or continuous).
    pub emission_rate: M2AnimationBlock<f32>,
    
    /// Random variation added to the base emission rate value.
    /// Introduced in WotLK (264).
    pub emission_rate_variation: Option<f32>, 
    
    /// For plane generators: width of the emission area. For sphere generators: maximum radius.
    pub emission_area_width: M2AnimationBlock<f32>,
    /// For plane generators: length of the emission area. For sphere generators: minimum radius.
    pub emission_area_length: M2AnimationBlock<f32>,
    /// When greater than 0, defines the initial velocity Z source offset.
    pub z_source: M2AnimationBlock<f32>,

    // ==========================================
    // FAKE BLOCKS (FBlock)
    // ==========================================
    /// Particle color track. Usually contains 3 timestamps for {start, middle, end}.
    pub color_track: FakeAnimationBlock<C3Vector>,
    /// Particle alpha/opacity track. Stored as fixed16 (i16) in the binary.
    pub alpha_track: FakeAnimationBlock<i16>,
    /// Particle scale track.
    pub scale_track: FakeAnimationBlock<C2Vector>,
    
    /// A percentage amount to randomly vary the scale of each particle.
    pub scale_vary: C2Vector, 
    
    /// UV animation sequence for the head particle's life.
    pub head_uv_anim: FakeAnimationBlock<u16>,
    /// UV animation sequence for the tail particle's life.
    pub tail_uv_anim: FakeAnimationBlock<u16>,

    // ==========================================
    // PHYSICAL & RENDER PARAMETERS
    // ==========================================
    /// A multiplier to the calculated tail particle length.
    pub tail_length: f32,
    /// Blinking speed of the particle.
    pub twinkle_speed: f32,
    /// How visible the particle is (1.0 = 100% of the time, 0.5 = 50% of the time).
    pub twinkle_percent: f32,
    /// Minimum scale variation for the twinkle effect.
    pub twinkle_scale_min: f32,
    /// Maximum scale variation for the twinkle effect.
    pub twinkle_scale_max: f32,
    /// Scales the velocity inherited from the parent particle.
    pub inherit_velocity_scale: f32,
    /// Particles slow down over time. Speed is multiplied by exp(-drag * time).
    pub drag: f32,

    // ==========================================
    // SPIN (Rotation)
    // ==========================================
    /// Spin value for Vanilla/TBC (0.0 for none, 1.0 for full 360 degrees rotation).
    pub legacy_spin: Option<f32>, 
    
    /// Initial rotation of the particle quad (WotLK+).
    pub base_spin: Option<f32>,
    /// Variation of the initial rotation (WotLK+).
    pub base_spin_variation: Option<f32>,
    /// Rotation of the particle quad per second (WotLK+).
    pub spin_speed: Option<f32>,
    /// Variation of the rotation speed (WotLK+).
    pub spin_speed_variation: Option<f32>,

    // ==========================================
    // VECTORS & SPLINES
    // ==========================================
    /// Minimum angular velocity (3D model particle rotation).
    pub tumble_min: C3Vector,
    /// Maximum angular velocity (3D model particle rotation).
    pub tumble_max: C3Vector,
    /// Static wind parameters, ignored if the DynamicWind flag is set.
    pub wind_vector: C3Vector,
    /// Wind time factor.
    pub wind_time: f32,

    /// Follow speed multiplier 1.
    pub follow_speed1: f32,
    /// Follow scale multiplier 1.
    pub follow_scale1: f32,
    /// Follow speed multiplier 2.
    pub follow_speed2: f32,
    /// Follow scale multiplier 2.
    pub follow_scale2: f32,

    /// Array of points for spline. Set only for spline particle emitters.
    pub spline_points: M2Array<C3Vector>, 
    
    /// Boolean track linking particles to animation sets where they are enabled.
    pub enabled_in: M2AnimationBlock<u8>,

    // ==========================================
    // MODERN EXTENSIONS (WoD, Legion, BfA+)
    // ==========================================
    /// Explicit fallback model if main one fails to load (Legion+)
    pub fallback_model_filename: Option<M2Array<u8>>,
    /// Texture file IDs for multi-texture particles (Legion+)
    pub texture_file_data_ids: Option<M2Array<u32>>,
    /// Flag to enable encryption (WoD+)
    pub enable_encryption: Option<u8>,
    /// Multi-texture particle blend operation (BfA+)
    pub multi_texture_param0: Option<[u8; 4]>,
    /// Multi-texture particle blend flags (BfA+)
    pub multi_texture_param1: Option<[u8; 4]>,
    /// Base initial state for particles (Legion+)
    pub particle_initial_state: Option<u32>,
    /// Variation for initial state (Legion+)
    pub particle_initial_state_variation: Option<f32>,
    /// Convergence speed for particles (Legion+)
    pub particle_convergence_time: Option<f32>,
}

impl M2ParticleEmitter {
    /// Parse a particle emitter from a reader based on the M2 version
    pub fn parse<R: Read + Seek>(reader: &mut R, version: u32) -> Result<Self> {
        let id = reader.read_u32_le()?;
        let flag_bits = reader.read_u32_le()?;
        let flags = M2ParticleFlags::from_bits_retain(flag_bits);
        let position = C3Vector::parse(reader)?;
        
        let bone_index = reader.read_u16_le()?;
        let texture_index_raw = reader.read_u16_le()?;

        let texture_indices = if version >= 272 {
            // Cataclysm+
            [
                texture_index_raw & 0x1F,
                (texture_index_raw >> 5) & 0x1F,
                (texture_index_raw >> 10) & 0x1F
            ]
        } else { [texture_index_raw, 0, 0] };
        
        let model_filename = M2Array::parse(reader)?;
        let child_emitter_filename = M2Array::parse(reader)?;

        // Legion+
        let m2_version = M2Version::from_header_version(version);
        let fallback_model_filename = if let Some(v) = m2_version {
            if v >= M2Version::Legion {
                Some(M2Array::parse(reader)?)
            } else { None }
        } else { None };

        let blending_type;
        let emitter_type;
        let particle_color_index;

        if version >= 262 {
            // TBC+
            blending_type = reader.read_u8()?;
            emitter_type = M2ParticleEmitterType::from_u8(reader.read_u8()?).unwrap_or(M2ParticleEmitterType::Point);
            particle_color_index = Some(reader.read_u16_le()?);
        } else { 
            // Vanilla, they are u16 ! (total 4 bytes)
            blending_type = reader.read_u16_le()? as u8;
            emitter_type = M2ParticleEmitterType::from_u8(reader.read_u16_le()? as u8).unwrap_or(M2ParticleEmitterType::Point);
            particle_color_index = None;
        }

        let mut particle_type = 0;
        let mut head_or_tail = 0;
        let mut multi_tex_scale = None;

        if version >= 272 {
            // Cataclysm+
            // 2 bytes has multiTexScale (int8)
            let scale_0 = reader.read_i8()?;
            let scale_1 = reader.read_i8()?;
            multi_tex_scale = Some([scale_0, scale_1]);
        } else {
            // 2 bytes with particleType and headOrTail (uint8)
            particle_type = reader.read_u8()?;
            head_or_tail = reader.read_u8()?;
        }

        // --- EXTENSIONS MODERNES (WoD, Legion, BfA) ---
        let texture_file_data_ids = if let Some(v) = m2_version {
            if v >= M2Version::Legion {
                Some(M2Array::parse(reader)?)
            } else { None }
        } else { None };

        let enable_encryption = if let Some(v) = m2_version {
            if v >= M2Version::WoD {
                Some(reader.read_u8()?)
            } else { None }
        } else { None };

        let (multi_texture_param0, multi_texture_param1) = if let Some(v) = m2_version {
            if v >= M2Version::BfA {
                let mut p0 = [0u8; 4];
                let mut p1 = [0u8; 4];
                reader.read_exact(&mut p0)?;
                reader.read_exact(&mut p1)?;
                (Some(p0), Some(p1))
            } else { (None, None) }
        } else { (None, None) };

        // =========================================
        // COMMON BLOCK
        // ==========================================
        // --- OFFSETS 0x2E to 0x33: PriorityPlane, Rows, Columns ---
        let priority_plane = reader.read_u16_le()?;
        let texture_dimensions_rows = reader.read_u16_le()?;
        let texture_dimensions_columns = reader.read_u16_le()?;

        // --- OFFSETS 0x34: M2Tracks (Fail-Safe) ---
        let emission_speed = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
        let speed_variation = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
        let vertical_range = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
        let horizontal_range = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
        let gravity = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
        let lifespan = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();

        // Read common parameters
        let lifespan_variation = if version >= 264 {
            reader.read_f32_le().ok()
        } else {
            None
        };

        let emission_rate = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
        let emission_rate_variation = if version >= 264 {
            reader.read_f32_le().ok()
        } else {
            None
        };

        let emission_area_width = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
        let emission_area_length = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
        let z_source = M2AnimationBlock::<f32>::parse(reader, version).unwrap_or_default();
    
        let color_track;
        let alpha_track;
        let scale_track;
        let scale_vary;
        let head_uv_anim;
        let tail_uv_anim;

        let empty_c2 = C2Vector { x: 1.0, y: 1.0 }; // Default scale
        let empty_u16 = M2Array::new(0, 0);
        let empty_fblock_vec3 = FakeAnimationBlock { timestamps: empty_u16.clone(), values: M2Array::new(0, 0) };
        let empty_fblock_i16 = FakeAnimationBlock { timestamps: empty_u16.clone(), values: M2Array::new(0, 0) };
        let empty_fblock_vec2 = FakeAnimationBlock { timestamps: empty_u16.clone(), values: M2Array::new(0, 0) };
        let empty_fblock_u16 = FakeAnimationBlock { timestamps: empty_u16.clone(), values: M2Array::new(0, 0) };

        if version >= 264 {
            // WotLK (Wrath)
            color_track = FakeAnimationBlock::<C3Vector>::parse(reader).unwrap_or(empty_fblock_vec3);
            alpha_track = FakeAnimationBlock::<i16>::parse(reader).unwrap_or(empty_fblock_i16);
            scale_track = FakeAnimationBlock::<C2Vector>::parse(reader).unwrap_or(empty_fblock_vec2);
            scale_vary = C2Vector::parse(reader).unwrap_or(empty_c2);
            head_uv_anim = FakeAnimationBlock::<u16>::parse(reader).unwrap_or(empty_fblock_u16.clone());
            tail_uv_anim = FakeAnimationBlock::<u16>::parse(reader).unwrap_or(empty_fblock_u16);
        } else {
            // Vanilla & TBC (< 264)
            // We move cursor 48 bytes forward. Fail-safe.
            let mut buf = [0u8; 48];
            let _ = reader.read_exact(&mut buf);
            
            color_track = empty_fblock_vec3;
            alpha_track = empty_fblock_i16;
            scale_track = empty_fblock_vec2;
            scale_vary = empty_c2;
            head_uv_anim = empty_fblock_u16.clone();
            tail_uv_anim = empty_fblock_u16;
        };

        // =========================================
        // INCONDITIONNAL BLOCK
        // ==========================================
        let tail_length = reader.read_f32_le().unwrap_or(0.0);
        let twinkle_speed = reader.read_f32_le().unwrap_or(0.0);
        let twinkle_percent = reader.read_f32_le().unwrap_or(0.0);
        let twinkle_scale_min = reader.read_f32_le().unwrap_or(0.0);
        let twinkle_scale_max = reader.read_f32_le().unwrap_or(0.0);
        let inherit_velocity_scale = reader.read_f32_le().unwrap_or(0.0);
        let drag = reader.read_f32_le().unwrap_or(0.0);

        // ==========================================
        // SPIN BIFURCATION (Wrath vs Vanilla/TBC)
        // ==========================================
        let legacy_spin;
        let base_spin;
        let base_spin_variation;
        let spin_speed;
        let spin_speed_variation;

        if version >= 264 { // WotLK
            base_spin = reader.read_f32_le().ok();
            base_spin_variation = reader.read_f32_le().ok();
            spin_speed = reader.read_f32_le().ok();
            spin_speed_variation = reader.read_f32_le().ok();
            legacy_spin = None;
        } else { // Vanilla & TBC
            legacy_spin = reader.read_f32_le().ok();
            base_spin = None;
            base_spin_variation = None;
            spin_speed = None;
            spin_speed_variation = None;
        }

        let empty_c3 = C3Vector { x: 0.0, y: 0.0, z: 0.0 };
        
        let tumble_min = C3Vector::parse(reader).unwrap_or(empty_c3);
        let tumble_max = C3Vector::parse(reader).unwrap_or(empty_c3);
        let wind_vector = C3Vector::parse(reader).unwrap_or(empty_c3);
        let wind_time = reader.read_f32_le().unwrap_or(0.0);
        
        let follow_speed1 = reader.read_f32_le().unwrap_or(0.0);
        let follow_scale1 = reader.read_f32_le().unwrap_or(0.0);
        let follow_speed2 = reader.read_f32_le().unwrap_or(0.0);
        let follow_scale2 = reader.read_f32_le().unwrap_or(0.0);
        
        let spline_points = M2Array::<C3Vector>::parse(reader).unwrap_or(M2Array::new(0, 0));
        let enabled_in = M2AnimationBlock::<u8>::parse(reader, version).unwrap_or_default();

        // --- Legion+ ---
        let (particle_initial_state, particle_initial_state_variation, particle_convergence_time) = if let Some(v) = m2_version {
            if v >= M2Version::Legion {
                (
                    Some(reader.read_u32_le()?),
                    Some(reader.read_f32_le()?),
                    Some(reader.read_f32_le()?),
                )
            } else { (None, None, None) }
        } else { (None, None, None) };

        Ok(Self {
            id,
            flags,
            position,
            bone_index,
            texture_indices,
            model_filename,
            child_emitter_filename: Some(child_emitter_filename),
            blending_type,
            emitter_type,
            particle_color_index,
            particle_type,
            head_or_tail,
            priority_plane: priority_plane as i16,
            texture_dimensions_rows,
            texture_dimensions_columns,

            emission_speed,
            speed_variation,
            vertical_range,
            horizontal_range,
            gravity,
            lifespan,
            lifespan_variation,
            emission_rate,
            emission_rate_variation,
            emission_area_width,
            emission_area_length,
            z_source,

            color_track,
            alpha_track,
            scale_track,
            scale_vary,
            head_uv_anim,
            tail_uv_anim,

            tail_length,
            twinkle_speed,
            twinkle_percent,
            twinkle_scale_min,
            twinkle_scale_max,
            inherit_velocity_scale,
            drag,

            legacy_spin,
            base_spin,
            base_spin_variation,
            spin_speed,
            spin_speed_variation,

            tumble_min,
            tumble_max,
            wind_vector,
            wind_time,

            follow_speed1,
            follow_scale1,
            follow_speed2,
            follow_scale2,

            spline_points,
            enabled_in

            fallback_model_filename: None,
            texture_file_data_ids: None,
            enable_encryption: None,
            multi_texture_param0: None,
            multi_texture_param1: None,
            particle_initial_state: None,
            particle_initial_state_variation: None,
            particle_convergence_time: None,
        })
    }

    /// Write a particle emitter to a writer based on the M2 version
    pub fn write<W: Write>(&self, writer: &mut W, version: u32) -> Result<()> {writer.write_u32_le(self.id)?;
        writer.write_u32_le(self.flags.bits())?;
        self.position.write(writer)?;
        writer.write_u16_le(self.bone_index)?;

        // Texture indices packing
        let texture_index_raw = if version >= 272 {
            self.texture_indices[0] | (self.texture_indices[1] << 5) | (self.texture_indices[2] << 10)
        } else {
            self.texture_indices[0]
        };
        writer.write_u16_le(texture_index_raw)?;

        self.model_filename.write(writer)?;
        
        if let Some(ref child) = self.child_emitter_filename {
            child.write(writer)?;
        } else {
            crate::common::M2Array::<u8>::new(0, 0).write(writer)?;
        }

        if version >= 262 {
            writer.write_u8(self.blending_type)?;
            writer.write_u8(self.emitter_type as u8)?;
            writer.write_u16_le(self.particle_color_index.unwrap_or(0))?;
        } else {
            writer.write_u16_le(self.blending_type as u16)?;
            writer.write_u16_le(self.emitter_type as u16)?;
        }

        if version >= 272 {
            // Write default scales if Cataclysm+ (since we didn't preserve multi_tex_scale perfectly)
            writer.write_i8(0)?;
            writer.write_i8(0)?;
        } else {
            writer.write_u8(self.particle_type)?;
            writer.write_u8(self.head_or_tail)?;
        }

        writer.write_u16_le(self.priority_plane as u16)?;
        writer.write_u16_le(self.texture_dimensions_rows)?;
        writer.write_u16_le(self.texture_dimensions_columns)?;

        // Animation blocks (M2Track)
        self.emission_speed.write(writer, version)?;
        self.speed_variation.write(writer, version)?;
        self.vertical_range.write(writer, version)?;
        self.horizontal_range.write(writer, version)?;
        self.gravity.write(writer, version)?;
        self.lifespan.write(writer, version)?;

        if version >= 264 {
            writer.write_f32_le(self.lifespan_variation.unwrap_or(0.0))?;
        }

        self.emission_rate.write(writer, version)?;

        if version >= 264 {
            writer.write_f32_le(self.emission_rate_variation.unwrap_or(0.0))?;
        }

        self.emission_area_width.write(writer, version)?;
        self.emission_area_length.write(writer, version)?;
        self.z_source.write(writer, version)?;

        // Fake Blocks (FBlock)
        if version >= 264 {
            self.color_track.write(writer)?;
            self.alpha_track.write(writer)?;
            self.scale_track.write(writer)?;
            self.scale_vary.write(writer)?;
            self.head_uv_anim.write(writer)?;
            self.tail_uv_anim.write(writer)?;
        } else {
            // Write the 48 bytes gap for pre-WotLK to maintain offset integrity
            writer.write_all(&[0u8; 48])?;
        }

        // Unconditional block
        writer.write_f32_le(self.tail_length)?;
        writer.write_f32_le(self.twinkle_speed)?;
        writer.write_f32_le(self.twinkle_percent)?;
        writer.write_f32_le(self.twinkle_scale_min)?;
        writer.write_f32_le(self.twinkle_scale_max)?;
        writer.write_f32_le(self.inherit_velocity_scale)?;
        writer.write_f32_le(self.drag)?;

        // Spin Bifurcation
        if version >= 264 {
            writer.write_f32_le(self.base_spin.unwrap_or(0.0))?;
            writer.write_f32_le(self.base_spin_variation.unwrap_or(0.0))?;
            writer.write_f32_le(self.spin_speed.unwrap_or(0.0))?;
            writer.write_f32_le(self.spin_speed_variation.unwrap_or(0.0))?;
        } else {
            writer.write_f32_le(self.legacy_spin.unwrap_or(0.0))?;
        }

        // Vectors & Splines
        self.tumble_min.write(writer)?;
        self.tumble_max.write(writer)?;
        self.wind_vector.write(writer)?;
        writer.write_f32_le(self.wind_time)?;

        writer.write_f32_le(self.follow_speed1)?;
        writer.write_f32_le(self.follow_scale1)?;
        writer.write_f32_le(self.follow_speed2)?;
        writer.write_f32_le(self.follow_scale2)?;

        self.spline_points.write(writer)?;
        self.enabled_in.write(writer, version)?;

        Ok(())
    }

    /// Convert this particle emitter to a different version
    pub fn convert(&self, target_version: M2Version) -> Self {
        let mut new_emitter = self.clone();

        // Handle version-specific conversions
        if target_version >= M2Version::Legion && self.fallback_model_filename.is_none() {
            // When upgrading to Legion or later, add fallback model filename and texture file data IDs if missing
            new_emitter.fallback_model_filename = Some(M2Array::new(0, 0));
            new_emitter.texture_file_data_ids = Some(M2Array::new(0, 0));
        } else if target_version < M2Version::Legion {
            // When downgrading to pre-Legion, remove fallback model filename and texture file data IDs
            new_emitter.fallback_model_filename = None;
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

        if target_version >= M2Version::Legion && self.particle_initial_state.is_none() {
            // When upgrading to Legion or later, add particle state if missing
            new_emitter.particle_initial_state = Some(0);
            new_emitter.particle_initial_state_variation = Some(0.0);
            new_emitter.particle_convergence_time = Some(0.0);
        } else if target_version < M2Version::Legion {
            // When downgrading to pre-Legion, remove particle state
            new_emitter.particle_initial_state = None;
            new_emitter.particle_initial_state_variation = None;
            new_emitter.particle_convergence_time = None;
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
