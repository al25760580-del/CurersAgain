//! Explicit-path, read-only native save inspector. Prints structure, not personal values.
fn main() -> Result<(), String> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("Usage: native_save_inspect /path/to/save_n.dat")?;
    let save = holocure_core_port::native_save::NativeSave::load(std::path::Path::new(&path))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&save.summary()).map_err(|e| e.to_string())?
    );
    Ok(())
}
