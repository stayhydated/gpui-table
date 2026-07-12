use std::path::Path;

/// Converts a `file!()` source path like
/// `examples/some-lib/src/structs/user.rs` into a use-path like
/// `some_lib::structs::user` for the glob import at the top of each generated file.
pub fn source_path_to_use_path(source_path: &str) -> Option<syn::Path> {
    let path = Path::new(source_path);
    let components: Vec<_> = path.components().collect();

    let src_index = components
        .iter()
        .position(|c| matches!(c, std::path::Component::Normal(s) if s.to_str() == Some("src")))?;

    if src_index == 0 {
        return None;
    }
    let crate_name = match &components[src_index - 1] {
        std::path::Component::Normal(s) => s.to_str()?.replace('-', "_"),
        _ => return None,
    };

    let mut path_segments = vec![crate_name];
    for component in &components[src_index + 1..] {
        if let std::path::Component::Normal(s) = component {
            let segment = s.to_str()?;
            if segment == "mod.rs" {
                continue;
            }
            path_segments.push(
                segment
                    .strip_suffix(".rs")
                    .unwrap_or(segment)
                    .replace('-', "_"),
            );
        }
    }

    syn::parse_str(&path_segments.join("::")).ok()
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
}
