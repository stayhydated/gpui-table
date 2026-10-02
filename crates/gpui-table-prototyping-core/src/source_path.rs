use std::path::{Component, Path};

/// Converts a `file!()` source path like
/// `examples/some-lib/src/structs/user.rs` into a use-path like
/// `some_lib::structs::user` for the glob import at the top of each generated file.
pub fn source_path_to_use_path(source_path: &str) -> Option<syn::Path> {
    let mut components = Path::new(source_path).components();
    let mut crate_component = None;
    loop {
        let component = components.next()?;
        if component == Component::Normal("src".as_ref()) {
            break;
        }
        crate_component = Some(component);
    }

    let Some(Component::Normal(crate_name)) = crate_component else {
        return None;
    };
    let mut use_path = crate_name.to_str()?.replace('-', "_");

    for component in components {
        let Component::Normal(segment) = component else {
            continue;
        };
        let segment = segment.to_str()?;
        if segment == "mod.rs" {
            continue;
        }
        use_path.push_str("::");
        use_path.push_str(
            &segment
                .strip_suffix(".rs")
                .unwrap_or(segment)
                .replace('-', "_"),
        );
    }

    syn::parse_str(&use_path).ok()
}

#[cfg(test)]
mod tests {
    use super::source_path_to_use_path;
    use quote::ToTokens as _;

    fn parsed(path: &str) -> Option<String> {
        source_path_to_use_path(path).map(|path| path.into_token_stream().to_string())
    }

    #[test]
    fn source_paths_become_crate_qualified_rust_paths() {
        assert_eq!(
            parsed("examples/some-lib/src/structs/user.rs").as_deref(),
            Some("some_lib :: structs :: user")
        );
        assert_eq!(
            parsed("my-crate/src/nested/mod.rs").as_deref(),
            Some("my_crate :: nested")
        );
        assert_eq!(
            parsed("my-crate/src/lib.rs").as_deref(),
            Some("my_crate :: lib")
        );
    }

    #[test]
    fn paths_without_a_crate_before_src_are_rejected() {
        assert_eq!(parsed("src/user.rs"), None);
        assert_eq!(parsed("user.rs"), None);
    }

    #[test]
    fn path_conversion_preserves_the_first_source_root_and_normalizes_segments() {
        assert_eq!(
            parsed("examples/my-crate/src/nested-dir/src/user.rs").as_deref(),
            Some("my_crate :: nested_dir :: src :: user")
        );
        assert_eq!(
            parsed("my-crate/src/../mod.rs").as_deref(),
            Some("my_crate")
        );
        assert_eq!(parsed("my-crate/not-src/user.rs"), None);
        assert_eq!(parsed("my-crate/src/invalid name.rs"), None);
        assert_eq!(parsed("src/src/user.rs"), None);
    }
}
