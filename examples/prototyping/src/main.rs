use gpui_table::registry::GpuiTableShape;
use gpui_table_prototyping_core::code_gen::generate_table_story;
use heck::ToSnakeCase as _;
use std::{fs, path::Path};

// Import target lib to trigger inventory registrations
#[allow(unused_imports)]
use some_lib::*;

fn main() {
    let output_dir = &Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");
    println!("Generating table stories in: {}", output_dir.display());

    for table_shape in inventory::iter::<GpuiTableShape>() {
        println!("Generating story for: {:?}", table_shape.struct_name);

        let syn_file = generate_table_story(table_shape);
        let struct_snake_case_name = table_shape.struct_name.to_snake_case();
        let file_path = output_dir.join(format!("{}.rs", struct_snake_case_name));

        let formatted_code = prettyplease::unparse(&syn_file);

        fs::write(&file_path, formatted_code)
            .unwrap_or_else(|_| panic!("Failed to write file: {}", file_path.display()));

        println!("Generated and formatted: {}", file_path.display());
    }

    println!("Table story generation complete.");
}
