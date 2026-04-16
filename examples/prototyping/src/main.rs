use anyhow::Context as _;
use gpui_table::registry::GpuiTableShape;
use gpui_table_prototyping_core::{TableLayout, TableParts, TableShapeAdapter};
use heck::ToSnakeCase as _;
use quote::quote;
use std::{collections::BTreeSet, fs, path::Path};

// import targeted lib to get inventory registrations
extern crate some_lib;

struct StorybookLayout;

impl TableLayout for StorybookLayout {
    fn generate_file(&self, parts: &TableParts) -> syn::File {
        let TableParts {
            story_struct_ident,
            imports,
            delegate_creation,
            table_state_creation,
            field_initializers,
            struct_fields,
            render_children,
            title_expr,
            ..
        } = parts;

        syn::parse2(quote! {
            #imports

            #[gpui_storybook::story_init]
            pub fn init(_cx: &mut App) {}

            #[gpui_storybook::story]
            pub struct #story_struct_ident {
                #struct_fields
            }

            impl gpui_storybook::Story for #story_struct_ident {
                fn title() -> String {
                    #title_expr
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
                    #delegate_creation
                    #table_state_creation

                    Self {
                        #field_initializers
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
                        #render_children
                }
            }
        })
        .expect("Failed to parse generated tokens into syn::File for table story")
    }
}

fn main() -> anyhow::Result<()> {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "failed to create output directory `{}`",
            output_dir.display()
        )
    })?;
    println!("Generating table stories in: {}", output_dir.display());

    let mut modules: BTreeSet<String> = BTreeSet::new();

    for shape in inventory::iter::<GpuiTableShape>() {
        println!("Table: {:?}", shape.struct_name);

        let syn_file = TableShapeAdapter::new(shape, true)
            .try_generate_file(&StorybookLayout)
            .with_context(|| {
                format!("failed to generate table story for `{}`", shape.struct_name)
            })?;
        let file_stem = shape.struct_name.to_snake_case();
        let file_path = output_dir.join(format!("{file_stem}.rs"));

        fs::write(&file_path, prettyplease::unparse(&syn_file))
            .with_context(|| format!("failed to write `{}`", file_path.display()))?;

        modules.insert(file_stem);
        println!("Generated and formatted: {}", file_path.display());
    }

    let mod_rs_path = output_dir.join("mod.rs");
    let mod_rs = modules
        .iter()
        .map(|m| format!("pub mod {m};\n"))
        .collect::<String>();

    fs::write(&mod_rs_path, mod_rs)
        .with_context(|| format!("failed to write `{}`", mod_rs_path.display()))?;

    println!("Generated module index: {}", mod_rs_path.display());
    println!("Table story generation complete.");
    Ok(())
}
