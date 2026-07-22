use std::fs;
use std::io::Cursor;
use std::path::Path;
use wow_m2::parse_m2;

fn test_model(path: &Path) -> anyhow::Result<()> {
    println!("\n═══════════════════════════════════════════════════════");
    println!("Testing: {}", path.display());
    println!("═══════════════════════════════════════════════════════");

    // Read the M2 file
    let data = fs::read(path)?;
    println!("File size: {} bytes", data.len());

    // Parse the M2 model
    let mut cursor = Cursor::new(&data);
    let m2_format = parse_m2(&mut cursor)?;
    let model = m2_format.model();

    // Display basic header info
    println!("\n📊 Model Header:");
    println!("  Version: {}", model.header.version);
    println!("  Name: {:?}", model.header.name);
    println!("  Flags: 0x{:x}", model.header.flags);

    // Extract enhanced data
    println!("\n📊 Extracting comprehensive model data...");
    let enhanced_data = model.parse_all_data(&data)?;

    // Display comprehensive information
    model.display_info(&enhanced_data);

    // TBC-specific validation
    println!("\n✅ TBC-Specific Validation:");

    // Check version (TBC uses 260-263)
    if model.header.version >= 260 && model.header.version <= 263 {
        println!("  ✓ Version {} is valid TBC version", model.header.version);
    } else {
        println!("  ⚠ Version {} is not a typical TBC version", model.header.version);
    }

    // Check for embedded skins (TBC still has them)
    if model.has_embedded_skins() {
        println!("  ✓ Has embedded skins (correct for TBC)");

        // Parse embedded skins
        let mut skin_count = 0;
        for i in 0..4 {
            // TBC models typically have up to 4 skins
            if let Ok(skin) = model.parse_embedded_skin(&data, i) {
                skin_count += 1;
                println!(
                    "    Skin {}: {} indices, {} triangles, {} submeshes",
                    i,
                    skin.indices().len(),
                    skin.triangles().len(),
                    skin.submeshes().len()
                );
            } else {
                break;
            }
        }
        if skin_count > 0 {
            println!("  ✓ Successfully parsed {} embedded skins", skin_count);
        }
    } else {
        println!("  ⚠ No embedded skins found (unexpected for TBC)");
    }

    // Check vertices
    if !enhanced_data.vertices.is_empty() {
        println!("  ✓ Vertices parsed: {}", enhanced_data.vertices.len());

        // Sample first few vertices
        println!("\n  Sample vertices (first 3):");
        for (i, vertex) in enhanced_data.vertices.iter().take(3).enumerate() {
            println!(
                "    Vertex {}: pos=({:.2}, {:.2}, {:.2}), uv=({:.2}, {:.2})",
                i, vertex.position.x, vertex.position.y, vertex.position.z, vertex.tex_coords.x, vertex.tex_coords.y
            );
        }
    }

    // Check bones
    if !enhanced_data.bones.is_empty() {
        println!("\n  ✓ Bones parsed: {}", enhanced_data.bones.len());

        let root_bones = enhanced_data.bones.iter().filter(|b| b.bone.parent_bone == -1).count();
        println!("  ✓ Root bones: {}", root_bones);
    }

    // Check animations
    if !enhanced_data.animations.is_empty() {
        println!("\n  ✓ Animations parsed: {}", enhanced_data.animations.len());

        // Show first few animations
        println!("  Sample animations (first 5):");
        for (i, anim) in enhanced_data.animations.iter().take(5).enumerate() {
            println!(
                "    Anim {}: duration={}ms, looping={}",
                i, anim.duration_ms, anim.is_looping
            );
        }
    }

    // Check textures
    if !enhanced_data.textures.is_empty() {
        println!("\n  ✓ Textures parsed: {}", enhanced_data.textures.len());
        for (i, tex) in enhanced_data.textures.iter().enumerate() {
            println!(
                "    Texture {}: type={:?}, flags=0x{:x}",
                i, tex.texture_type, tex.texture.flags
            );
        }
    }

    println!("\n✅ TBC model successfully parsed and validated!");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║       M2 Enhanced Parser - TBC (2.4.3) Test          ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    // Test with TBC HumanMale model
    let tbc_model = Path::new(
        "/home/danielsreichenbach/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/2.4.3/m2/HumanMale.M2",
    );

    if !tbc_model.exists() {
        println!("\n⚠️  TBC model not found: {}", tbc_model.display());
        println!("Please ensure the sample data is available.");
        return Ok(());
    }

    match test_model(tbc_model) {
        Ok(_) => {
            println!("\n🎉 TBC model test completed successfully!");
        }
        Err(e) => {
            println!("\n❌ Error testing TBC model: {}", e);
            return Err(e);
        }
    }

    // Also test with vanilla model for comparison
    println!("\n\n═══════════════════════════════════════════════════════");
    println!("Testing vanilla model for comparison...");
    println!("═══════════════════════════════════════════════════════");

    let vanilla_model = Path::new(
        "/home/danielsreichenbach/Repos/github.com/wowemulation-dev/blender-wow-addon/sample_data/1.12.1/m2/HumanMale.m2",
    );

    if vanilla_model.exists() {
        match test_model(vanilla_model) {
            Ok(_) => println!("\n✅ Vanilla model also parsed successfully for comparison"),
            Err(e) => println!("\n⚠️  Vanilla model error (for reference): {}", e),
        }
    }

    Ok(())
}
