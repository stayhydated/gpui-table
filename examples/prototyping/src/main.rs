#![allow(unused)]

use gpui_table::registry::GpuiTableShape;
use gpui_table_prototyping_core::code_gen::{TableIdentities as _, TableShape, TableShapeAdapter};
use heck::ToSnakeCase as _;

use quote::quote;
use std::{collections::BTreeSet, fs, path::Path};

// Import target lib to trigger inventory registrations
#[allow(unused_imports)]
use some_lib::*;

fn source_path_to_use_path(source_path: &str) -> Option<syn::Path> {
    let path = Path::new(source_path);
    let components: Vec<_> = path.components().collect();

    let src_index = components
        .iter()
        .position(|c| matches!(c, std::path::Component::Normal(s) if s.to_str() == Some("src")))?;

    if src_index == 0 {
        return None;
    }
    let crate_component = &components[src_index - 1];
    let crate_name = match crate_component {
        std::path::Component::Normal(s) => s.to_str()?.replace('-', "_"),
        _ => return None,
    };

    let mut path_segments = vec![crate_name];

    for component in &components[src_index + 1..] {
        if let std::path::Component::Normal(s) = component {
            let segment = s.to_str()?;
            let segment = segment.strip_suffix(".rs").unwrap_or(segment);
            path_segments.push(segment.replace('-', "_"));
        }
    }

    let path_str = path_segments.join("::");
    syn::parse_str(&path_str).ok()
}

fn main() {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    println!("Generating table stories in: {}", output_dir.display());

    let mut modules: BTreeSet<String> = BTreeSet::new();

    for table_shape in inventory::iter::<GpuiTableShape>() {
        println!("Table: {:?}", table_shape.struct_name);
        let syn_file = layout(table_shape);
        let struct_snake_case_name = table_shape.struct_name.to_snake_case();
        let file_path = output_dir.join(format!("{}.rs", struct_snake_case_name));

        let formatted_code = prettyplease::unparse(&syn_file);

        fs::write(&file_path, formatted_code)
            .unwrap_or_else(|_| panic!("Failed to write file: {}", file_path.display()));

        modules.insert(struct_snake_case_name);

        println!("Generated and formatted: {}", file_path.display());
    }

    let mod_rs_path = output_dir.join("mod.rs");
    let mut mod_rs = String::new();

    for m in modules {
        mod_rs.push_str(&format!("pub mod {m};\n"));
    }

    fs::write(&mod_rs_path, mod_rs)
        .unwrap_or_else(|_| panic!("Failed to write file: {}", mod_rs_path.display()));

    println!("Generated module index: {}", mod_rs_path.display());
    println!("Table story generation complete.");
}

fn layout(data: &GpuiTableShape) -> syn::File {
    // Generate with all_filters() helper for cleaner code (use_filter_helpers = true)
    let adapter = TableShapeAdapter::new(data, true);

    // Access identities for various idents
    let struct_name_ident = adapter.identities.struct_name_ident();
    let story_struct_ident = adapter.identities.story_struct_ident();

    // Build import path from source_path
    let source_module_path = source_path_to_use_path(data.source_path)
        .unwrap_or_else(|| panic!("Failed to parse source_path: {}", data.source_path));
    let target_types_import = quote! {
        use #source_module_path::*;
    };

    // Get code generation components from the adapter
    let delegate_creation_tokens = adapter.delegate_creation();
    let table_state_creation_tokens = adapter.table_state_creation();
    let field_initializers_tokens = adapter.field_initializers();
    let struct_fields_tokens = adapter.struct_fields();
    let render_children_tokens = adapter.render_children();
    let title_expr_tokens = adapter.title_expr();

    let import_tokens = quote! {
        #target_types_import
        use gpui::{
            App, AppContext as _, Context, Entity, Focusable, IntoElement,
            ParentElement, Render, Styled, Subscription, Window,
        };
        use gpui_component::{
            h_flex,
            table::{Table, TableState},
            v_flex,
        };
        use es_fluent::ThisFtl as _;
    };

    let layout_tokens = quote! {
        #import_tokens

        #[gpui_storybook::story_init]
        pub fn init(_cx: &mut App) {}

        #[gpui_storybook::story]
        pub struct #story_struct_ident {
            #struct_fields_tokens
        }

        impl gpui_storybook::Story for #story_struct_ident {
            fn title() -> String {
                #title_expr_tokens
            }

            fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
                Self::view(window, cx)
            }
        }

        impl Focusable for #story_struct_ident {
            fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
                self.table.focus_handle(cx)
            }
        }

        impl #story_struct_ident {
            pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
                cx.new(|cx| Self::new(window, cx))
            }

            fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
                #delegate_creation_tokens
                #table_state_creation_tokens

                Self {
                    #field_initializers_tokens
                }
            }
        }

        impl Render for #story_struct_ident {
            fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let table = self.table.read(cx);
                let delegate = table.delegate();

                v_flex()
                    .size_full()
                    .gap_4()
                    .p_4()
                    #render_children_tokens
            }
        }
    };

    syn::parse2(layout_tokens)
        .expect("Failed to parse generated tokens into syn::File for table story")
}
