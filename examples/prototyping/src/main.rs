use gpui_table::registry::GpuiTableShape;
use gpui_table_prototyping_core::code_gen::{TableIdentities as _, TableShape, TableShapeAdapter};
use heck::ToSnakeCase as _;

use quote::quote;
use std::{fs, path::Path};

// Import target lib to trigger inventory registrations
#[allow(unused_imports)]
use some_lib::*;

fn main() {
    let output_dir = &Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");
    println!("Generating table stories in: {}", output_dir.display());

    for table_shape in inventory::iter::<GpuiTableShape>() {
        println!("Table: {:?}", table_shape.struct_name);
        let syn_file = layout(table_shape);
        let struct_snake_case_name = table_shape.struct_name.to_snake_case();
        let file_path = output_dir.join(format!("{}.rs", struct_snake_case_name));

        let formatted_code = prettyplease::unparse(&syn_file);

        fs::write(&file_path, formatted_code)
            .unwrap_or_else(|_| panic!("Failed to write file: {}", file_path.display()));

        println!("Generated and formatted: {}", file_path.display());
    }
    println!("Table story generation complete.");
}

fn layout(data: &GpuiTableShape) -> syn::File {
    let adapter = TableShapeAdapter::new(data);

    // Access identities for various idents
    let struct_name_ident = adapter.identities.struct_name_ident();
    let story_struct_ident = adapter.identities.story_struct_ident();
    let story_id_literal = adapter.identities.story_id_literal();

    // Build import path - customize this for your library
    let struct_name_path_qualifier = adapter.identities.snake_case_ident();
    let target_types_import = quote! {
        use some_lib::structs::#struct_name_path_qualifier::*;
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
        use fake::Fake;
        use gpui::{
            App, AppContext, Context, Entity, Focusable, IntoElement,
            ParentElement, Render, Styled, Window,
        };
        use gpui_component::{
            table::{Table, TableState, TableDelegate as _},
            v_flex,
        };
        use es_fluent::ToFluentString as _;
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
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let table = &self.table.read(cx);
                let delegate = table.delegate();
                let rows_count = delegate.rows_count(cx);

                v_flex()
                    .size_full()
                    .text_sm()
                    .gap_4()
                    #render_children_tokens
            }
        }
    };

    syn::parse2(layout_tokens)
        .expect("Failed to parse generated tokens into syn::File for table story")
}
