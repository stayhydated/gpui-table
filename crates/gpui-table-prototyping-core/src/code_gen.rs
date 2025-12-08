//! Code generation utilities for table view scaffolds.

use gpui_table_core::registry::GpuiTableShape;
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Identities derived from a table shape.
pub struct TableIdentities<'a>(&'a GpuiTableShape);

impl<'a> TableIdentities<'a> {
    pub fn new(shape: &'a GpuiTableShape) -> Self {
        Self(shape)
    }

    /// The original struct name (e.g., "User")
    pub fn struct_name(&self) -> &'static str {
        self.0.struct_name
    }

    /// The struct name as an identifier
    pub fn struct_name_ident(&self) -> syn::Ident {
        syn::parse_str(self.struct_name()).unwrap()
    }

    /// The table story struct name (e.g., "UserTableStory")
    pub fn story_struct_ident(&self) -> syn::Ident {
        format_ident!("{}TableStory", self.struct_name())
    }

    /// The table delegate struct name (e.g., "UserTableDelegate")
    pub fn delegate_struct_ident(&self) -> syn::Ident {
        format_ident!("{}TableDelegate", self.struct_name())
    }

    /// The table ID
    pub fn table_id(&self) -> &'static str {
        self.0.table_id
    }

    /// The snake_case version of struct name for file paths
    pub fn snake_case_name(&self) -> String {
        self.struct_name().to_snake_case()
    }

    /// Fluent label enum identifier (e.g., "UserLabelKvFtl")
    pub fn ftl_label_ident(&self) -> syn::Ident {
        format_ident!("{}LabelKvFtl", self.struct_name())
    }
}

/// Adapter for generating code from a table shape.
pub struct TableShapeAdapter<'a> {
    pub shape: &'a GpuiTableShape,
    pub identities: TableIdentities<'a>,
}

impl<'a> TableShapeAdapter<'a> {
    pub fn new(shape: &'a GpuiTableShape) -> Self {
        Self {
            shape,
            identities: TableIdentities::new(shape),
        }
    }

    /// Generate the full table story module code.
    pub fn generate_story_code(&self) -> TokenStream {
        let struct_name_ident = self.identities.struct_name_ident();
        let story_struct_ident = self.identities.story_struct_ident();
        let delegate_struct_ident = self.identities.delegate_struct_ident();
        let snake_case_name = self.identities.snake_case_name();
        let ftl_label_ident = self.identities.ftl_label_ident();

        let import_path = syn::parse_str::<syn::Ident>(&snake_case_name).unwrap();

        quote! {
            use some_lib::structs::#import_path::*;
            use fake::Fake;
            use gpui::{
                App, AppContext, Context, Entity, Focusable, IntoElement,
                ParentElement, Render, Styled, Window,
            };
            use gpui_component::{
                table::{Table, TableState},
                v_flex,
            };
            use es_fluent::ToFluentString as _;

            #[gpui_storybook::story_init]
            pub fn init(_cx: &mut App) {}

            #[gpui_storybook::story]
            pub struct #story_struct_ident {
                table: Entity<TableState<#delegate_struct_ident>>,
            }

            impl gpui_storybook::Story for #story_struct_ident {
                fn title() -> String {
                    #ftl_label_ident::this_ftl()
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
                    let mut delegate = #delegate_struct_ident::new(vec![]);
                    for _ in 0..100 {
                        delegate.rows.push(fake::Faker.fake());
                    }

                    let table = cx.new(|cx| TableState::new(delegate, window, cx));

                    Self { table }
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
                        .child(format!("Total Rows: {}", rows_count))
                        .child(Table::new(&self.table))
                }
            }
        }
    }
}

/// Generate a complete syn::File from the table shape.
pub fn generate_table_story(shape: &GpuiTableShape) -> syn::File {
    let adapter = TableShapeAdapter::new(shape);
    let tokens = adapter.generate_story_code();
    syn::parse2(tokens).expect("Failed to parse generated table story code")
}
